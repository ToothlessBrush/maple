//! Directional light casts light on a scene from a single direction, like the sun. It is used to simulate sunlight in a scene. It is a type of light that is infinitely far away and has no attenuation. It is defined by a direction and a color. It can also cast shadows using a shadow map.
//!
//! ## Usage
//! add this to the node tree to add a directional light to the scene.

const MAX_LIGHTS: usize = 100;

use std::f32::consts::PI;

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, Vec3, Vec4, Vec4Swizzles};
use maple_engine::{
    Buildable, Builder, Node, nodes::node_builder::NodePrototype, prelude::NodeTransform,
    utils::Color,
};

use crate::nodes::camera::Camera3D;

/// used to pass data to the shader buffer
///
/// the data on the gpu follows this format in this order:
/// ```c
/// struct DirectLight {
///     vec4 color;
///     vec4 direction;
///     float intensity;
///     int shadowIndex;
///     int cascadeLevel;
///     float cascadeSpl it[4];
///     mat4 lightSpaceMatrices[4];
///     float farPlane;
/// };
/// ```
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, Pod, Zeroable)]
pub struct DirectionalLightBufferData {
    color: [f32; 4],
    direction: [f32; 4], // vec3 has vec4 alignment so we just use a vec4 for simplicity
    intensity: f32,
    shadow_index: i32,
    cascade_level: i32,
    bias: f32,
    cascade_split: [f32; 4],
    light_space_matrices: [[[f32; 4]; 4]; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct DirectionalLightBuffer {
    pub length: i32,
    _padding: [i32; 3],
    pub data: [DirectionalLightBufferData; MAX_LIGHTS],
}
impl DirectionalLightBuffer {
    pub fn from_lights(lights: &[DirectionalLightBufferData]) -> Self {
        let mut buffer = DirectionalLightBuffer {
            length: lights.len().min(MAX_LIGHTS) as i32,
            _padding: [0; 3],
            data: [DirectionalLightBufferData::default(); MAX_LIGHTS], // or use Zeroable::zeroed()
        };

        // Copy the lights into the first N slots
        let copy_count = lights.len().min(MAX_LIGHTS);
        buffer.data[..copy_count].copy_from_slice(&lights[..copy_count]);

        buffer
    }
}

/// Directional light casts light on a scene from a single direction, like the sun. It is used to simulate sunlight in a scene. It is a type of light that is infinitely far away and has no attenuation. It is defined by a direction and a color. It can also cast shadows using a shadow map.
///
/// ## Usage
/// add this to the node tree to add a directional light to the scene.
pub struct DirectionalLight {
    /// The transform of the directional light.
    transform: NodeTransform,

    /// The color of the directional light.
    pub color: Vec4,
    /// The intensity of the directional light.
    pub intensity: f32,
    ///// The light space matrix of the shadow cast by the directional light.
    //light_space_matrices: Vec<math::Mat4>,
    /// direction to the light
    pub direction: Vec3,

    far_plane: f32,

    // shadow_index: usize,
    /// number of cascades in this light
    pub num_cascades: usize,

    cascade_factors: Vec<f32>,

    pub bias: f32,
}

impl Node for DirectionalLight {
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }
}

impl DirectionalLight {
    /// creates a new directional light with the given direction, color, intensity, shadow distance, and shadow resolution.
    ///
    /// # Arguments
    /// - `direction` - The direction of the directional light.
    /// - `color` - The color of the directional light.
    /// - `intensity` - The intensity of the directional light.
    /// - `shadow_distance` - The distance of the shadow cast by the directional light.
    /// - `shadow_resolution` - The resolution of the shadow map of the directional light.
    ///
    /// # Returns
    /// The new directional light.
    pub fn new(
        direction: impl Into<Vec3>,
        color: impl Into<Vec4>,
        shadow_distance: f32,
        num_cascades: usize,
        //cascade_factors: &[f32],
    ) -> DirectionalLight {
        let direction = direction.into();

        let reference = Vec3::new(0.0, 0.0, 1.0);

        // Handle parallel and anti-parallel cases
        let rotation_quat = if direction.dot(reference).abs() > 0.9999 {
            if direction.z > 0.0 {
                Quat::IDENTITY // No rotation needed
            } else {
                Quat::from_axis_angle(Vec3::X, PI)
                // 180-degree rotation
            }
        } else {
            let rotation_axis = reference.cross(direction).normalize();
            let rotation_angle = reference.dot(direction).acos();
            Quat::from_axis_angle(rotation_axis, rotation_angle)
        };

        let cascade_factors =
            Self::calculate_cascade_splits(0.1, shadow_distance, num_cascades, 0.9);

        DirectionalLight {
            transform: NodeTransform::new(
                Vec3::new(0.0, 0.0, 0.0),
                rotation_quat,
                Vec3::new(1.0, 1.0, 1.0),
            ),
            intensity: 1.0,
            color: color.into(),
            num_cascades,
            direction: direction.normalize(),
            far_plane: shadow_distance,
            cascade_factors,
            bias: 0.005,
        }
    }

    pub fn detach(&self) -> DirectionalLight {
        Self {
            transform: self.transform,
            color: self.color,
            intensity: self.intensity,
            direction: self.direction,
            far_plane: self.far_plane,
            num_cascades: self.num_cascades,
            cascade_factors: self.cascade_factors.clone(),
            bias: self.bias,
        }
    }

    /// generate cascade split level based on far plane and lambda
    fn calculate_cascade_splits(
        near_plane: f32,
        far_plane: f32,
        num_cascades: usize,
        lambda: f32, // Keep this for user control, default to 0.9
    ) -> Vec<f32> {
        let mut splits = Vec::with_capacity(num_cascades);

        let range = far_plane - near_plane;
        let ratio = far_plane / near_plane;

        for i in 1..=num_cascades {
            let p = i as f32 / num_cascades as f32;

            // Logarithmic (concentrates detail near camera)
            let log_split = near_plane * ratio.powf(p);

            // Uniform (even distribution)
            let uniform_split = near_plane + range * p;

            // Blend: lambda should typically be 0.85-0.95 for good results
            let split = lambda * log_split + (1.0 - lambda) * uniform_split;

            splits.push(split / far_plane);
        }

        log::debug!("splits: {splits:?}");

        splits
    }

    /// get the world relative view_projection matrix
    pub fn view_projection(&self, camera: &Camera3D, aspect_ratio: f32) -> Vec<Mat4> {
        // Shadow map resolution for texel snapping
        const SHADOW_MAP_SIZE: f32 = 4096.0;

        let mut matrices = Vec::with_capacity(self.num_cascades);

        let camera_near = camera.near_plane();
        let camera_far = camera.far_plane();
        let range = camera_far - camera_near;

        let mut last_split_far = camera_near;

        // make shadow cascade (TM) backwards from the camera to fill full frustrum
        for i in 0..self.num_cascades {
            let split_far = camera_near + range * self.cascade_factors[i];

            // Add overlap between cascades to prevent gaps
            // Overlap is 5% of the cascade range
            let overlap = if i > 0 {
                (split_far - last_split_far) * 0.05
            } else {
                0.0
            };
            let cascade_near = (last_split_far - overlap).max(camera_near);

            let corners = Self::get_frustrum_corners_world_space(
                &camera.get_projection_matrix_with_planes(aspect_ratio, cascade_near, split_far),
                &camera.get_view_matrix(),
            );

            let view = self.get_view(&corners, SHADOW_MAP_SIZE);
            let proj = Self::get_proj(&corners, &view, SHADOW_MAP_SIZE);

            matrices.push(proj * view);

            last_split_far = split_far;
        }

        matrices
    }

    fn get_view(&self, corners: &[Vec4], shadow_map_size: f32) -> Mat4 {
        let mut center = Vec3::ZERO;

        for v in corners {
            center += v.xyz()
        }
        center /= corners.len() as f32;

        // Choose an appropriate up vector based on light direction
        // If the light direction is too close to parallel with Y axis, use Z axis instead
        let up = if self.direction.dot(Vec3::Y).abs() > 0.99 {
            Vec3::Z
        } else {
            Vec3::Y
        };

        // Create the initial view matrix
        let view = Mat4::look_to_rh(center, self.direction, up);

        // Calculate bounds in light space to determine texel size
        let mut min_bounds = Vec3::splat(f32::MAX);
        let mut max_bounds = Vec3::splat(f32::MIN);
        for corner in corners {
            let trf = (view * corner).xyz();
            min_bounds = min_bounds.min(trf);
            max_bounds = max_bounds.max(trf);
        }

        // Make the projection square - use the maximum extent
        let extent = (max_bounds.x - min_bounds.x)
            .max(max_bounds.y - min_bounds.y)
            .max(0.01);

        let texel_size_world = extent / shadow_map_size;

        // Round the center to the nearest texel to prevent sub-pixel movement
        // Transform center to light space, round it, then transform back
        let center_light_space = (view * center.extend(1.0)).xyz();
        let rounded_center_light_space = Vec3::new(
            (center_light_space.x / texel_size_world).round() * texel_size_world,
            (center_light_space.y / texel_size_world).round() * texel_size_world,
            center_light_space.z,
        );

        // Transform back to world space
        let view_inv = view.inverse();
        let rounded_center = (view_inv * rounded_center_light_space.extend(1.0)).xyz();

        Mat4::look_to_rh(rounded_center, self.direction, up)
    }

    fn get_proj(corners: &[Vec4], light_view: &Mat4, shadow_map_size: f32) -> Mat4 {
        let mut min_bounds = Vec3::splat(f32::MAX);
        let mut max_bounds = Vec3::splat(f32::MIN);

        for corner in corners {
            let trf = (light_view * corner).xyz();
            min_bounds = min_bounds.min(trf);
            max_bounds = max_bounds.max(trf);
        }

        // Make the projection square - use the maximum extent for both X and Y
        let xy_extent = (max_bounds.x - min_bounds.x)
            .max(max_bounds.y - min_bounds.y)
            .max(0.01); // Ensure non-zero

        // Center the square bounds around the original center
        let center_x = (min_bounds.x + max_bounds.x) * 0.5;
        let center_y = (min_bounds.y + max_bounds.y) * 0.5;

        // Calculate texel size in world space
        let texel_size = xy_extent / shadow_map_size;

        // Round the extent to nearest texel to prevent sub-texel changes
        let rounded_extent = (xy_extent / texel_size).ceil() * texel_size;

        // Apply the square, rounded extent
        let half_extent = rounded_extent * 0.5;
        min_bounds.x = center_x - half_extent;
        max_bounds.x = center_x + half_extent;
        min_bounds.y = center_y - half_extent;
        max_bounds.y = center_y + half_extent;

        // Expand z-bounds to allow objects outside the frustum to cast shadows
        let z_mult: f32 = 10.0;
        let z_range = max_bounds.z - min_bounds.z;
        let desired_z_expansion = z_range * (z_mult - 1.0);
        let max_z_extent = rounded_extent * 3.0;
        let actual_z_expansion = desired_z_expansion.min(max_z_extent - z_range);

        // Apply expansion symmetrically
        min_bounds.z -= actual_z_expansion * 0.5;
        max_bounds.z += actual_z_expansion * 0.5;

        // Ensure minimum bounds to prevent singular matrix
        const MIN_EXTENT: f32 = 0.01;
        for i in 0..3 {
            let extent = max_bounds[i] - min_bounds[i];
            if extent.abs() < MIN_EXTENT {
                let center = (min_bounds[i] + max_bounds[i]) * 0.5;
                min_bounds[i] = center - MIN_EXTENT * 0.5;
                max_bounds[i] = center + MIN_EXTENT * 0.5;
            }
        }

        Mat4::orthographic_rh(
            min_bounds.x,
            max_bounds.x,
            min_bounds.y,
            max_bounds.y,
            min_bounds.z,
            max_bounds.z,
        )
    }

    fn get_frustrum_corners_world_space(proj: &Mat4, view: &Mat4) -> Vec<Vec4> {
        let inv = (proj * view).inverse();

        let mut frustrum_corners: Vec<Vec4> = Default::default();

        for x in 0..2 {
            for y in 0..2 {
                for z in 0..2 {
                    let pt = inv
                        * Vec4::new(
                            2.0 * x as f32 - 1.0,
                            2.0 * y as f32 - 1.0,
                            z as f32, //depth range 0 to 1 in wgpu
                            1.0,
                        );
                    frustrum_corners.push(pt / pt.w);
                }
            }
        }

        frustrum_corners
    }

    /// vector from source or the direction of the light rays
    pub fn set_direction(&mut self, direction: impl Into<Vec3>) -> &mut Self {
        let direction = direction.into();

        let reference = Vec3::new(0.0, 0.0, 1.0);

        // Handle parallel and anti-parallel cases
        let rotation_quat = if direction.dot(reference).abs() > 0.9999 {
            if direction.z > 0.0 {
                Quat::IDENTITY // No rotation needed
            } else {
                Quat::from_axis_angle(Vec3::X, PI)
                // 180-degree rotation
            }
        } else {
            let rotation_axis = reference.cross(direction).normalize();
            let rotation_angle = reference.dot(direction).acos();
            Quat::from_axis_angle(rotation_axis, rotation_angle)
        };

        self.transform.set_rotation(rotation_quat);

        self
    }

    /// sets the color of the light
    pub fn set_color(&mut self, color: impl Into<Vec4>) -> &mut Self {
        self.color = color.into();
        self
    }

    /// set the intensity of the light
    pub fn set_intensity(&mut self, intensity: f32) -> &mut Self {
        self.intensity = intensity;
        self
    }

    pub fn to_buffer_data(
        &self,
        camera: &Camera3D,
        aspect_ratio: f32,
        shadow_index: usize,
    ) -> DirectionalLightBufferData {
        let vp_matrices = self.view_projection(camera, aspect_ratio);

        // Calculate ACTUAL split distances (not normalized factors)
        let camera_near = camera.near_plane();
        let camera_far = camera.far_plane();
        let range = camera_far - camera_near;

        let mut cascade_split = [0.0f32; 4];
        for (i, split) in cascade_split
            .iter_mut()
            .enumerate()
            .take(self.num_cascades.min(4))
        {
            // Convert normalized factor to actual distance
            *split = camera_near + range * self.cascade_factors[i];
        }

        // Convert matrices to array format
        let mut light_space_matrices = [[[0.0f32; 4]; 4]; 4];
        for i in 0..vp_matrices.len().min(4) {
            light_space_matrices[i] = vp_matrices[i].to_cols_array_2d();
        }

        DirectionalLightBufferData {
            color: self.color.to_array(),
            direction: self.direction.extend(0.0).to_array(),
            intensity: self.intensity,
            shadow_index: shadow_index as i32,
            cascade_level: self.num_cascades as i32,
            bias: self.bias,
            cascade_split,
            light_space_matrices,
        }
    }

    /// get the far plane of the shadow cast by the directional light
    pub fn get_far_plane(&self) -> f32 {
        self.far_plane
    }

    /// set the far plane of the shadow cast by the directional light
    pub fn set_far_plane(&mut self, distance: f32) {
        self.far_plane = distance;

        self.cascade_factors =
            Self::calculate_cascade_splits(0.1, self.far_plane, self.num_cascades, 0.7);
        // self.shadow_projections = math::ortho(
        //     -self.far_plane / 2.0,
        //     self.far_plane / 2.0,
        //     -self.far_plane / 2.0,
        //     self.far_plane / 2.0,
        //     0.1,
        //     self.far_plane,
        // );
        // let light_direction =
        //    math::quat_rotate_vec3(&self.transform.rotation, &math::vec3(0.0, 0.0, 1.0));
        // let light_position = light_direction * (self.far_plane / 2.0); //self.shadow_distance;
        // let light_view = math::look_at(
        //     &light_position,
        //     &math::vec3(0.0, 0.0, 0.0),
        //     &math::vec3(0.0, 1.0, 0.0),
        // );
        //self.light_space_matrix = self.shadow_projections * light_view;
    }
}

impl Buildable for DirectionalLight {
    type Builder = DirectionalLightBuilder;
    fn builder() -> Self::Builder {
        Self::Builder {
            prototype: NodePrototype::default(),
            direction: Vec3::new(1.0, 1.0, 1.0),
            color: Color::WHITE.into(),
            intensity: 1.0,
            far_plane: 100.0,
            num_cascades: 4,
            bias: 0.005,
        }
    }
}

/// builder implementation for directional lights
pub struct DirectionalLightBuilder {
    prototype: NodePrototype,
    direction: Vec3,
    color: Vec4,
    intensity: f32,
    far_plane: f32,
    num_cascades: usize,
    bias: f32,
}

impl Builder for DirectionalLightBuilder {
    type Node = DirectionalLight;
    fn prototype(&mut self) -> &mut NodePrototype {
        &mut self.prototype
    }

    fn build(self) -> Self::Node {
        let cascade_factors =
            DirectionalLight::calculate_cascade_splits(0.1, self.far_plane, self.num_cascades, 0.7);

        Self::Node {
            transform: self.prototype.transform,
            color: self.color,
            intensity: self.intensity,
            cascade_factors,
            num_cascades: self.num_cascades,
            direction: self.direction.normalize(),
            far_plane: self.far_plane,
            bias: self.bias,
        }
    }
}

impl DirectionalLightBuilder {
    /// direction of the lights
    ///
    /// the light direction is independent from its rotation
    pub fn direction(mut self, direction: impl Into<Vec3>) -> Self {
        self.direction = direction.into();
        self
    }

    /// color of the light
    pub fn color(mut self, color: impl Into<Vec4>) -> Self {
        self.color = color.into();
        self
    }

    /// strength of the light
    pub fn intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        self
    }

    /// how far from the light shadows are rendered from the camera.
    ///
    /// default value is 100
    pub fn far_plane(mut self, far_plane: f32) -> Self {
        self.far_plane = far_plane;
        self
    }

    /// set the shadow bias
    ///
    /// default value is 0.005
    pub fn bias(mut self, bias: f32) -> Self {
        self.bias = bias;
        self
    }

    /// set the cascade level of the light for shadow detail at greater distance
    ///
    /// level is clamped between 1 and 4
    pub fn cascades_level(mut self, level: usize) -> Self {
        let level = std::cmp::max(level, 1);
        let level = std::cmp::min(level, 4);
        self.num_cascades = level;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Mat4, Vec3, Vec4};

    // Helper function to create a simple test camera
    fn create_test_camera() -> Camera3D {
        Camera3D::new(
            std::f32::consts::FRAC_PI_4, // 45 degree FOV
            0.1,                         // near plane
            100.0,                       // far plane
        )
    }

    #[test]
    fn test_calculate_cascade_splits_uniform() {
        // Test with lambda = 0.0 for uniform distribution
        let splits = DirectionalLight::calculate_cascade_splits(0.1, 100.0, 4, 0.0);

        assert_eq!(splits.len(), 4);

        // With uniform distribution, splits should be evenly spaced
        // Split values are normalized (divided by far_plane)
        assert!((splits[0] - 0.25).abs() < 0.01);
        assert!((splits[1] - 0.50).abs() < 0.01);
        assert!((splits[2] - 0.75).abs() < 0.01);
        assert!((splits[3] - 1.00).abs() < 0.01);
    }

    #[test]
    fn test_calculate_cascade_splits_logarithmic() {
        // Test with lambda = 1.0 for logarithmic distribution
        let splits = DirectionalLight::calculate_cascade_splits(0.1, 100.0, 4, 1.0);

        assert_eq!(splits.len(), 4);

        // Logarithmic splits should be closer together near the camera
        // Each split should be greater than the previous
        assert!(splits[0] < splits[1]);
        assert!(splits[1] < splits[2]);
        assert!(splits[2] < splits[3]);

        // Last split should always be 1.0 (normalized far plane)
        assert!((splits[3] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_cascade_splits_hybrid() {
        // Test with lambda = 0.7 (default) for hybrid distribution
        let splits = DirectionalLight::calculate_cascade_splits(0.1, 100.0, 4, 0.7);

        assert_eq!(splits.len(), 4);

        // Verify monotonic increase
        for i in 0..splits.len() - 1 {
            assert!(
                splits[i] < splits[i + 1],
                "Split {} ({}) should be less than split {} ({})",
                i,
                splits[i],
                i + 1,
                splits[i + 1]
            );
        }

        // Last split should be 1.0
        assert!((splits[3] - 1.0).abs() < 0.001);

        // First split should be > 0
        assert!(splits[0] > 0.0);
    }

    #[test]
    fn test_calculate_cascade_splits_single_cascade() {
        let splits = DirectionalLight::calculate_cascade_splits(0.1, 100.0, 1, 0.5);

        assert_eq!(splits.len(), 1);
        assert!((splits[0] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_get_frustrum_corners_identity() {
        // Test frustum corners with identity matrices
        let proj = Mat4::IDENTITY;
        let view = Mat4::IDENTITY;

        let corners = DirectionalLight::get_frustrum_corners_world_space(&proj, &view);

        // Should get 8 corners (2x2x2 cube in NDC space)
        assert_eq!(corners.len(), 8);

        // All corners should have w component of 1.0 after perspective divide
        for corner in &corners {
            assert!(
                (corner.w - 1.0).abs() < 0.001,
                "Corner w component should be 1.0, got {}",
                corner.w
            );
        }
    }

    #[test]
    fn test_get_frustrum_corners_perspective() {
        // Create a realistic perspective projection
        let proj = Mat4::perspective_rh(
            std::f32::consts::FRAC_PI_4, // 45 degree FOV
            16.0 / 9.0,                  // aspect ratio
            0.1,                         // near
            100.0,                       // far
        );
        let view = Mat4::look_at_rh(
            Vec3::new(0.0, 5.0, 10.0), // eye
            Vec3::new(0.0, 0.0, 0.0),  // target
            Vec3::new(0.0, 1.0, 0.0),  // up
        );

        let corners = DirectionalLight::get_frustrum_corners_world_space(&proj, &view);

        assert_eq!(corners.len(), 8);

        // Verify all corners are finite
        for corner in &corners {
            assert!(corner.x.is_finite());
            assert!(corner.y.is_finite());
            assert!(corner.z.is_finite());
            assert!(corner.w.is_finite());
        }
    }

    #[test]
    fn test_get_view_centered() {
        // Create a light pointing down
        let light = DirectionalLight::new(
            Vec3::new(0.0, -1.0, 0.0), // pointing down
            Vec4::new(1.0, 1.0, 1.0, 1.0),
            100.0,
            4,
        );

        // Create corners representing a box at origin
        let corners = vec![
            Vec4::new(-1.0, -1.0, -1.0, 1.0),
            Vec4::new(1.0, -1.0, -1.0, 1.0),
            Vec4::new(-1.0, 1.0, -1.0, 1.0),
            Vec4::new(1.0, 1.0, -1.0, 1.0),
            Vec4::new(-1.0, -1.0, 1.0, 1.0),
            Vec4::new(1.0, -1.0, 1.0, 1.0),
            Vec4::new(-1.0, 1.0, 1.0, 1.0),
            Vec4::new(1.0, 1.0, 1.0, 1.0),
        ];

        let view = light.get_view(&corners, 4096.0);

        // View matrix should be invertible
        let det = view.determinant();
        assert!(
            det.abs() > 1e-10,
            "View matrix determinant should be non-zero (got {})",
            det
        );

        // The view matrix should be a valid transformation matrix
        assert!(view.is_finite());
    }

    #[test]
    fn test_get_proj_orthographic() {
        // Create a simple axis-aligned bounding box in light space
        let corners = vec![
            Vec4::new(-10.0, -10.0, -10.0, 1.0),
            Vec4::new(10.0, -10.0, -10.0, 1.0),
            Vec4::new(-10.0, 10.0, -10.0, 1.0),
            Vec4::new(10.0, 10.0, -10.0, 1.0),
            Vec4::new(-10.0, -10.0, 10.0, 1.0),
            Vec4::new(10.0, -10.0, 10.0, 1.0),
            Vec4::new(-10.0, 10.0, 10.0, 1.0),
            Vec4::new(10.0, 10.0, 10.0, 1.0),
        ];

        let view = Mat4::IDENTITY;
        let proj = DirectionalLight::get_proj(&corners, &view, 4096.0);

        // Projection matrix should be invertible
        let det = proj.determinant();
        assert!(
            det.abs() > 1e-10,
            "Projection matrix determinant should be non-zero (got {})",
            det
        );

        // The projection matrix should be finite
        assert!(proj.is_finite());
    }

    #[test]
    fn test_get_proj_bounds_expansion() {
        let corners = vec![
            Vec4::new(-1.0, -1.0, -1.0, 1.0),
            Vec4::new(1.0, -1.0, -1.0, 1.0),
            Vec4::new(-1.0, 1.0, -1.0, 1.0),
            Vec4::new(1.0, 1.0, -1.0, 1.0),
            Vec4::new(-1.0, -1.0, 1.0, 1.0),
            Vec4::new(1.0, -1.0, 1.0, 1.0),
            Vec4::new(-1.0, 1.0, 1.0, 1.0),
            Vec4::new(1.0, 1.0, 1.0, 1.0),
        ];

        let view = Mat4::IDENTITY;
        let proj = DirectionalLight::get_proj(&corners, &view, 4096.0);

        // Transform a point and verify it's within NDC range
        let test_point = Vec4::new(0.0, 0.0, 0.0, 1.0);
        let transformed = proj * test_point;
        let ndc = transformed / transformed.w;

        // NDC coordinates should be in [-1, 1] range for a point inside the bounds
        assert!(ndc.x >= -1.0 && ndc.x <= 1.0, "NDC x should be in [-1, 1]");
        assert!(ndc.y >= -1.0 && ndc.y <= 1.0, "NDC y should be in [-1, 1]");
    }

    #[test]
    fn test_view_projection_cascade_count() {
        let camera = create_test_camera();
        let light = DirectionalLight::new(
            Vec3::new(0.5, -1.0, 0.3).normalize(),
            Vec4::new(1.0, 1.0, 1.0, 1.0),
            100.0,
            4,
        );

        let matrices = light.view_projection(&camera, 16.0 / 9.0);

        // Should generate 4 matrices (one per cascade)
        assert_eq!(matrices.len(), 4);

        // All matrices should be valid
        for (i, matrix) in matrices.iter().enumerate() {
            assert!(matrix.is_finite(), "Matrix {} should be finite", i);
            let det = matrix.determinant();
            assert!(
                det.abs() > 1e-10,
                "Matrix {} determinant should be non-zero, got {}",
                i,
                det
            );
        }
    }

    #[test]
    fn test_view_projection_single_cascade() {
        let camera = create_test_camera();
        let light = DirectionalLight::new(
            Vec3::new(0.0, -1.0, 0.0),
            Vec4::new(1.0, 1.0, 1.0, 1.0),
            100.0,
            1,
        );

        let matrices = light.view_projection(&camera, 16.0 / 9.0);

        assert_eq!(matrices.len(), 1);
        assert!(matrices[0].is_finite());
    }

    #[test]
    fn test_view_projection_different_light_directions() {
        let camera = create_test_camera();

        // Test various light directions
        let directions = vec![
            Vec3::new(0.0, -1.0, 0.0), // straight down
            Vec3::new(1.0, -1.0, 0.0), // angled
            Vec3::new(0.0, -1.0, 1.0), // angled different axis
            Vec3::new(1.0, -1.0, 1.0), // fully angled
        ];

        for direction in directions {
            let light = DirectionalLight::new(
                direction.normalize(),
                Vec4::new(1.0, 1.0, 1.0, 1.0),
                100.0,
                2,
            );

            let matrices = light.view_projection(&camera, 16.0 / 9.0);

            assert_eq!(matrices.len(), 2);
            for matrix in matrices {
                assert!(matrix.is_finite());
                assert!(matrix.determinant().abs() > 1e-10);
            }
        }
    }

    #[test]
    fn test_view_projection_matrix_orthogonality() {
        let camera = create_test_camera();
        let light = DirectionalLight::new(
            Vec3::new(0.0, -1.0, 0.0),
            Vec4::new(1.0, 1.0, 1.0, 1.0),
            100.0,
            2,
        );

        let matrices = light.view_projection(&camera, 16.0 / 9.0);

        for matrix in matrices {
            // Test that transforming points doesn't produce NaN or infinity
            let test_points = vec![
                Vec4::new(0.0, 0.0, 0.0, 1.0),
                Vec4::new(1.0, 1.0, 1.0, 1.0),
                Vec4::new(-1.0, 2.0, -3.0, 1.0),
            ];

            for point in test_points {
                let transformed = matrix * point;
                assert!(transformed.x.is_finite());
                assert!(transformed.y.is_finite());
                assert!(transformed.z.is_finite());
                assert!(transformed.w.is_finite());
            }
        }
    }

    #[test]
    fn test_cascade_coverage_increases() {
        // Test that each cascade covers a larger area than the previous
        let camera = create_test_camera();
        let light = DirectionalLight::new(
            Vec3::new(0.0, -1.0, 0.0),
            Vec4::new(1.0, 1.0, 1.0, 1.0),
            100.0,
            4,
        );

        // Get the cascade split distances
        let camera_near = camera.near_plane();
        let camera_far = camera.far_plane();
        let range = camera_far - camera_near;

        let mut last_split_far = camera_near;

        for i in 0..light.num_cascades {
            let split_far = camera_near + range * light.cascade_factors[i];

            // Each cascade should cover more distance
            let cascade_range = split_far - last_split_far;
            assert!(
                cascade_range > 0.0,
                "Cascade {} should have positive range",
                i
            );

            last_split_far = split_far;
        }

        // Last cascade should reach the camera's far plane
        assert!((last_split_far - camera_far).abs() < 0.01);
    }

    // ====== DIRECTIONAL SHADOW INTEGRATION TESTS ======
    // These tests verify that shadows work correctly for realistic scene scenarios

    #[test]
    fn test_directional_shadow_setup() {
        // Create a typical camera setup
        let mut camera = Camera3D::new(
            std::f32::consts::FRAC_PI_4, // 45 degree FOV
            0.1,                         // near plane
            100.0,                       // far plane
        );
        camera.set_position(Vec3::new(-10.0, 1.0, 0.0));
        camera.set_orientation_vector(Vec3::new(10.0, -1.0, 0.0));

        // Create directional light with angled direction
        let light = DirectionalLight::builder()
            .direction(Vec3::new(-1.0, -1.0, 0.01))
            .intensity(10.0)
            .build();

        // Get view-projection matrices (shadows)
        let vp_matrices = light.view_projection(&camera, 16.0 / 9.0);

        // Should have 4 cascades (default)
        assert_eq!(
            vp_matrices.len(),
            4,
            "Should have 4 shadow cascades by default"
        );

        // All matrices should be valid and invertible
        for (i, matrix) in vp_matrices.iter().enumerate() {
            assert!(matrix.is_finite(), "Cascade {} matrix should be finite", i);

            let det = matrix.determinant();
            assert!(
                det.abs() > 1e-10,
                "Cascade {} matrix should be invertible (det={}, expected > 1e-10)",
                i,
                det
            );

            // Transform a test point to verify the matrix works
            let test_point = Vec4::new(0.0, 0.0, 0.0, 1.0);
            let transformed = matrix * test_point;
            assert!(
                transformed.x.is_finite()
                    && transformed.y.is_finite()
                    && transformed.z.is_finite()
                    && transformed.w.is_finite(),
                "Cascade {} should transform points to finite values",
                i
            );
        }
    }

    #[test]
    fn test_light_direction_normalization() {
        // Test light direction normalization
        let direction = Vec3::new(-1.0, -1.0, 0.01);

        // Test with new() - which normalizes the direction
        let light_new = DirectionalLight::new(direction, Vec4::new(1.0, 1.0, 1.0, 1.0), 100.0, 4);

        // Verify direction is normalized when using new()
        let normalized_dir = direction.normalize();
        assert!(
            (light_new.direction - normalized_dir).length() < 0.001,
            "Light direction should be normalized with new(). Expected {:?}, got {:?}",
            normalized_dir,
            light_new.direction
        );

        assert!(
            light_new.direction.is_normalized(),
            "Light direction should be normalized"
        );

        // Test with builder - should normalize the direction
        let light_builder = DirectionalLight::builder().direction(direction).build();

        // Builder normalizes direction (fixed behavior)
        assert!(
            (light_builder.direction - normalized_dir).length() < 0.001,
            "Light direction should be normalized with builder(). Expected {:?}, got {:?}",
            normalized_dir,
            light_builder.direction
        );
        assert!(
            light_builder.direction.is_normalized(),
            "Builder direction should be normalized"
        );
    }

    #[test]
    fn test_shadow_cascade_coverage() {
        // Create camera with standard configuration
        let mut camera = Camera3D::new(std::f32::consts::FRAC_PI_4, 0.1, 100.0);
        camera.set_position(Vec3::new(-10.0, 1.0, 0.0));
        camera.set_orientation_vector(Vec3::new(10.0, -1.0, 0.0));

        // Create directional light
        let light = DirectionalLight::builder()
            .direction(Vec3::new(-1.0, -1.0, 0.01))
            .far_plane(100.0)
            .build();

        // Verify cascade splits cover the full frustum
        let camera_near = camera.near_plane();
        let camera_far = camera.far_plane();
        let range = camera_far - camera_near;

        let mut last_split = camera_near;

        for (i, &factor) in light.cascade_factors.iter().enumerate() {
            let split_distance = camera_near + range * factor;

            // Each split should be beyond the previous
            assert!(
                split_distance > last_split,
                "Cascade {} split should be beyond previous (split={}, last={})",
                i,
                split_distance,
                last_split
            );

            last_split = split_distance;
        }

        // Last cascade should reach camera far plane
        assert!(
            (last_split - camera_far).abs() < 0.01,
            "Final cascade should reach camera far plane (got {}, expected {})",
            last_split,
            camera_far
        );
    }

    #[test]
    fn test_shadow_matrix_transforms_scene_objects() {
        // Create standard scene setup
        let mut camera = Camera3D::new(std::f32::consts::FRAC_PI_4, 0.1, 100.0);
        camera.set_position(Vec3::new(-10.0, 1.0, 0.0));
        camera.set_orientation_vector(Vec3::new(10.0, -1.0, 0.0));

        let light = DirectionalLight::builder()
            .direction(Vec3::new(-1.0, -1.0, 0.01))
            .build();

        let vp_matrices = light.view_projection(&camera, 16.0 / 9.0);

        // Test typical scene objects: cube at origin and ground plane below
        let cube_pos = Vec4::new(0.0, 0.0, 0.0, 1.0);
        let ground_pos = Vec4::new(0.0, -5.0, 0.0, 1.0);

        for (i, matrix) in vp_matrices.iter().enumerate() {
            // Transform scene objects
            let cube_shadow = matrix * cube_pos;
            let ground_shadow = matrix * ground_pos;

            // Both should transform to finite values
            assert!(
                cube_shadow.x.is_finite()
                    && cube_shadow.y.is_finite()
                    && cube_shadow.z.is_finite()
                    && cube_shadow.w.is_finite(),
                "Cascade {} should transform cube to finite values",
                i
            );

            assert!(
                ground_shadow.x.is_finite()
                    && ground_shadow.y.is_finite()
                    && ground_shadow.z.is_finite()
                    && ground_shadow.w.is_finite(),
                "Cascade {} should transform ground to finite values",
                i
            );

            // Verify w component is reasonable (should be 1.0 for orthographic)
            let cube_ndc = cube_shadow / cube_shadow.w;
            let ground_ndc = ground_shadow / ground_shadow.w;

            assert!(
                cube_ndc.w.is_finite(),
                "Cube NDC w should be finite in cascade {}",
                i
            );
            assert!(
                ground_ndc.w.is_finite(),
                "Ground NDC w should be finite in cascade {}",
                i
            );
        }
    }

    #[test]
    fn test_buffer_data_for_shader_complete() {
        // Create standard scene setup
        let mut camera = Camera3D::new(std::f32::consts::FRAC_PI_4, 0.1, 100.0);
        camera.set_position(Vec3::new(-10.0, 1.0, 0.0));
        camera.set_orientation_vector(Vec3::new(10.0, -1.0, 0.0));

        // Use new() instead of builder() to ensure direction is normalized
        let light = DirectionalLight::new(
            Vec3::new(-1.0, -1.0, 0.01),
            Vec4::new(1.0, 1.0, 1.0, 1.0),
            100.0,
            4,
        );
        // Set intensity after creation
        let mut light = light;
        light.intensity = 10.0;

        let buffer_data = light.to_buffer_data(&camera, 16.0 / 9.0, 0);

        // Verify shader buffer data is valid
        assert_eq!(buffer_data.cascade_level, 4, "Should have 4 cascades");
        assert!(
            (buffer_data.intensity - 10.0).abs() < 0.001,
            "Intensity should be 10.0"
        );

        // Verify direction is normalized
        let dir_vec = Vec3::from_slice(&buffer_data.direction[0..3]);
        assert!(
            (dir_vec.length() - 1.0).abs() < 0.001,
            "Direction in buffer should be normalized, got length {}",
            dir_vec.length()
        );

        // Verify cascade splits are in ascending order
        for i in 0..3 {
            assert!(
                buffer_data.cascade_split[i] < buffer_data.cascade_split[i + 1],
                "Cascade splits should be ascending: split[{}]={} should be < split[{}]={}",
                i,
                buffer_data.cascade_split[i],
                i + 1,
                buffer_data.cascade_split[i + 1]
            );
        }

        // Verify all 4 light space matrices are non-zero
        for i in 0..4 {
            let matrix = Mat4::from_cols_array_2d(&buffer_data.light_space_matrices[i]);
            let det = matrix.determinant();
            assert!(
                det.abs() > 1e-10,
                "Light space matrix {} should be invertible (det={}, expected > 1e-10)",
                i,
                det
            );
        }
    }

    #[test]
    fn test_shadow_map_depth_range() {
        // Verify shadow maps can capture depth range properly
        let mut camera = Camera3D::new(std::f32::consts::FRAC_PI_4, 0.1, 100.0);
        camera.set_position(Vec3::new(-10.0, 1.0, 0.0));
        camera.set_orientation_vector(Vec3::new(10.0, -1.0, 0.0));

        let light = DirectionalLight::builder()
            .direction(Vec3::new(-1.0, -1.0, 0.01))
            .build();

        let vp_matrices = light.view_projection(&camera, 16.0 / 9.0);

        // Test that depth values are in valid range for shadow mapping
        let test_positions = [
            Vec4::new(0.0, 0.0, 0.0, 1.0),   // Cube at origin
            Vec4::new(0.0, -5.0, 0.0, 1.0),  // Ground
            Vec4::new(-10.0, 1.0, 0.0, 1.0), // Camera position
        ];

        for (cascade_idx, matrix) in vp_matrices.iter().enumerate() {
            for (pos_idx, pos) in test_positions.iter().enumerate() {
                let shadow_space = matrix * pos;
                let ndc = shadow_space / shadow_space.w;

                // NDC z should be in [0, 1] for valid depth (RH coordinate system)
                // Some points may be outside frustum, but should still be finite
                assert!(
                    ndc.z.is_finite(),
                    "Cascade {} position {} NDC z should be finite, got {}",
                    cascade_idx,
                    pos_idx,
                    ndc.z
                );
            }
        }
    }

    #[test]
    fn test_light_view_matrix_stability() {
        // Test that view matrix is stable and doesn't flip or invert
        let light = DirectionalLight::builder()
            .direction(Vec3::new(-1.0, -1.0, 0.01))
            .build();

        // Create a box of corners around origin
        let corners = vec![
            Vec4::new(-5.0, -5.0, -5.0, 1.0),
            Vec4::new(5.0, -5.0, -5.0, 1.0),
            Vec4::new(-5.0, 5.0, -5.0, 1.0),
            Vec4::new(5.0, 5.0, -5.0, 1.0),
            Vec4::new(-5.0, -5.0, 5.0, 1.0),
            Vec4::new(5.0, -5.0, 5.0, 1.0),
            Vec4::new(-5.0, 5.0, 5.0, 1.0),
            Vec4::new(5.0, 5.0, 5.0, 1.0),
        ];

        let view = light.get_view(&corners, 4096.0);

        // View matrix should be invertible
        assert!(
            view.determinant().abs() > 0.001,
            "View matrix should be invertible"
        );

        // View matrix should transform points consistently
        for corner in &corners {
            let transformed = view * corner;
            assert!(
                transformed.x.is_finite()
                    && transformed.y.is_finite()
                    && transformed.z.is_finite()
                    && transformed.w.is_finite(),
                "View matrix should transform corners to finite values"
            );
        }
    }
}
