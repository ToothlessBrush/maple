//! Directional light casts light on a scene from a single direction, like the sun. It is used to simulate sunlight in a scene. It is a type of light that is infinitely far away and has no attenuation. It is defined by a direction and a color. It can also cast shadows using a shadow map.
//!
//! ## Usage
//! add this to the node tree to add a directional light to the scene.

const MAX_LIGHTS: usize = 100;

use std::f32::consts::PI;

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, Vec3, Vec4, Vec4Swizzles};
use maple_engine::{
    Buildable, Builder, Node, Scene,
    components::node_transform::WorldTransform,
    nodes::node_builder::NodePrototype,
    prelude::{EventReceiver, NodeTransform},
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
    far_plane: f32,
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
    /// The children of the directional light.
    children: Scene,

    events: EventReceiver,
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
}

impl Node for DirectionalLight {
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }

    fn get_children(&self) -> &Scene {
        &self.children
    }

    fn get_children_mut(&mut self) -> &mut Scene {
        &mut self.children
    }

    fn get_events(&mut self) -> &mut EventReceiver {
        &mut self.events
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
        direction: Vec3,
        color: impl Into<Vec4>,
        shadow_distance: f32,
        num_cascades: usize,
        //cascade_factors: &[f32],
    ) -> DirectionalLight {
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
            Self::calculate_cascade_splits(0.1, shadow_distance, num_cascades, 0.7);

        let mut light = DirectionalLight {
            transform: NodeTransform::new(
                Vec3::new(0.0, 0.0, 0.0),
                rotation_quat,
                Vec3::new(1.0, 1.0, 1.0),
            ),
            children: Scene::new(),
            events: EventReceiver::new(),
            intensity: 1.0,
            color: color.into(),
            num_cascades,
            direction: direction.normalize(),
            far_plane: shadow_distance,
            cascade_factors,
        };

        light
    }

    /// generate cascade split level based on far plane and lambda
    fn calculate_cascade_splits(
        near_plane: f32,
        far_plane: f32,
        num_cascades: usize,
        lambda: f32,
    ) -> Vec<f32> {
        let mut cascade_splits = Vec::with_capacity(num_cascades);

        for i in 1..=num_cascades {
            let uniform_split =
                near_plane + (far_plane - near_plane) * (i as f32 / num_cascades as f32);
            let log_split =
                near_plane * (far_plane / near_plane).powf(i as f32 / num_cascades as f32);
            let split = lambda * log_split + (1.0 - lambda) * uniform_split;
            cascade_splits.push(split / far_plane);
        }

        cascade_splits

        // return vec![0.01, 0.02, 0.03, 1.0]; // for testing
    }

    /// get the world relative view_projection matrix
    pub fn view_projection(&self, camera: &Camera3D, aspect_ratio: f32) -> Vec<Mat4> {
        let mut matrices = Vec::with_capacity(self.num_cascades);

        let camera_near = camera.near_plane();
        let camera_far = camera.far_plane();
        let range = camera_far - camera_near;

        let mut last_split_far = camera_near;

        // make shadow cascade (TM) backwards from the camera to fill full frustrum
        for i in 0..self.num_cascades {
            let split_far = camera_near + range * self.cascade_factors[i];

            let corners = Self::get_frustrum_corners_world_space(
                &camera.get_projection_matrix_with_planes(aspect_ratio, last_split_far, split_far),
                &camera.get_view_matrix(),
            );

            let view = self.get_view(&corners);
            let proj = Self::get_proj(&corners, &view);

            matrices.push(proj * view);

            last_split_far = split_far;
        }

        matrices
    }

    fn get_view(&self, corners: &[Vec4]) -> Mat4 {
        let mut center = Vec3::ZERO;

        for v in corners {
            center += v.xyz()
        }
        center /= corners.len() as f32;

        Mat4::look_to_rh(center, self.direction, Vec3::Y)
    }

    fn get_proj(corners: &[Vec4], light_view: &Mat4) -> Mat4 {
        let mut min_bounds = Vec3::splat(f32::MAX);
        let mut max_bounds = Vec3::splat(f32::MIN);

        for corner in corners {
            let trf = (light_view * corner).xyz();
            min_bounds = min_bounds.min(trf);
            max_bounds = max_bounds.max(trf);
        }

        // tune this value
        //
        // this affects how much more depth there is compared to the box so that objects that are
        // outside the frustrum can cast shadows
        let z_mult: f32 = 1.0;

        if min_bounds.z < 0.0 {
            min_bounds.z *= z_mult;
        } else {
            min_bounds.z /= z_mult;
        }
        if max_bounds.z < 0.0 {
            max_bounds.z *= z_mult;
        } else {
            max_bounds.z /= z_mult;
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
                            2.0 * z as f32 - 1.0,
                            1.0,
                        );
                    frustrum_corners.push(pt / pt.w);
                }
            }
        }

        frustrum_corners
    }

    /// vector from source or the direction of the light rays
    pub fn set_direction(&mut self, direction: Vec3) -> &mut Self {
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
    ) -> DirectionalLightBufferData {
        let vp_matrices = self.view_projection(camera, aspect_ratio);

        // Calculate ACTUAL split distances (not normalized factors)
        let camera_near = camera.near_plane();
        let camera_far = camera.far_plane();
        let range = camera_far - camera_near;

        let mut cascade_split = [0.0f32; 4];
        for i in 0..self.num_cascades.min(4) {
            // Convert normalized factor to actual distance
            cascade_split[i] = camera_near + range * self.cascade_factors[i];
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
            shadow_index: 0,
            cascade_level: self.num_cascades as i32,
            far_plane: camera_far,
            cascade_split,
            light_space_matrices,
        }
    }

    /// expand and array of mat4 to 3d array
    fn expand_matrix(vec: &[Mat4]) -> [[[f32; 4]; 4]; 4] {
        let mut arr = [[[0.0; 4]; 4]; 4];
        let len = vec.len().min(4);
        for (i, mat) in vec.iter().take(len).enumerate() {
            arr[i] = mat.to_cols_array_2d(); // Use glam's built-in method
        }
        arr
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
}

impl Builder for DirectionalLightBuilder {
    type Node = DirectionalLight;
    fn prototype(&mut self) -> &mut NodePrototype {
        &mut self.prototype
    }

    fn build(self) -> Self::Node {
        let cascade_factors =
            DirectionalLight::calculate_cascade_splits(0.1, self.far_plane, self.num_cascades, 0.7);

        let mut light = Self::Node {
            transform: self.prototype.transform,
            children: self.prototype.children,
            events: self.prototype.events,
            color: self.color,
            intensity: self.intensity,
            cascade_factors,
            num_cascades: self.num_cascades,
            direction: self.direction,
            far_plane: self.far_plane,
        };

        light
    }
}

impl DirectionalLightBuilder {
    /// direction of the lights
    ///
    /// the light direction is independent from its rotation
    pub fn direction(mut self, direction: Vec3) -> Self {
        self.direction = direction;
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
