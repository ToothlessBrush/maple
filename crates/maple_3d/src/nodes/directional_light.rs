//! Directional light casts light on a scene from a single direction, like the sun. It is used to simulate sunlight in a scene. It is a type of light that is infinitely far away and has no attenuation. It is defined by a direction and a color. It can also cast shadows using a shadow map.
//!
//! ## Usage
//! add this to the node tree to add a directional light to the scene.

const MAX_LIGHTS: usize = 100;

use std::f32::consts::PI;

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, Vec3, Vec4};
use maple_engine::{
    Buildable, Builder, Node, Scene,
    components::node_transform::WorldTransform,
    nodes::node_builder::NodePrototype,
    prelude::{EventReceiver, NodeTransform},
    utils::color::WHITE,
};

#[derive(Clone, Copy, Debug)]
struct Cascade {
    projection: Mat4,
}

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
    cascades: Vec<Cascade>,
    /// number of cascades in this light
    pub num_cascades: usize,

    cascade_factors: [f32; 4],
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
            cascades: Vec::default(),
            num_cascades,
            direction: direction.normalize(),
            far_plane: shadow_distance,
            cascade_factors,
        };

        light.gen_cascades(shadow_distance, num_cascades, cascade_factors.as_slice());

        light
    }

    /// generate cascade split level based on far plane and lambda
    fn calculate_cascade_splits(
        near_plane: f32,
        far_plane: f32,
        num_cascades: usize,
        lambda: f32,
    ) -> [f32; 4] {
        let mut cascade_splits = Vec::with_capacity(num_cascades);

        for i in 1..=num_cascades {
            let uniform_split =
                near_plane + (far_plane - near_plane) * (i as f32 / num_cascades as f32);
            let log_split =
                near_plane * (far_plane / near_plane).powf(i as f32 / num_cascades as f32);
            let split = lambda * log_split + (1.0 - lambda) * uniform_split;
            cascade_splits.push(split / far_plane);
        }
        let mut output = [1.0; 4];
        let len = cascade_splits.len().min(4);

        output[..len].copy_from_slice(&cascade_splits[..len]);

        output

        // return vec![0.01, 0.02, 0.03, 1.0]; // for testing
    }

    /// generate cascade matrices
    fn gen_cascades(&mut self, far_plane: f32, num_cascades: usize, cascade_factors: &[f32]) {
        let near_plane = 0.1;

        for i in 0..num_cascades {
            let radius = far_plane / 2.0 * cascade_factors.get(i).unwrap_or(&1.0);

            let projection =
                Mat4::orthographic_rh(-radius, radius, -radius, radius, near_plane, far_plane);

            // let direction = math::vec3(0.0, 0.0, 1.0);
            // let light_pos = math::normalize(&direction);

            // let view = math::look_at(
            //     &(light_pos * radius),
            //     &math::vec3(0.0, 0.0, 0.0),
            //     &math::vec3(0.0, 1.0, 0.0),
            // );

            self.cascades.push(Cascade { projection })
        }
    }

    /// get the world relative view_projection matrix
    ///
    /// # Arguments
    /// - 'location' - where to center the projection around (since shadow projections are centered around camera)
    ///
    /// # Returns
    /// the view_projection matrix
    pub fn view_projection(&self, location: &WorldTransform) -> Vec<Mat4> {
        let projection_offset = self.direction.normalize() * (self.far_plane / 2.0);
        let view = Mat4::look_at_rh(
            location.position + projection_offset,
            location.position,
            Vec3::new(0.0, 1.0, 0.0),
        );

        // println!("{:?}", view);

        // projection matrix doesnt change so we can just combine them to get the set of vp matrices
        self.cascades
            .iter()
            .map(|cascade| cascade.projection * view)
            .collect()
    }

    /// direction the lights coming from
    pub fn set_direction(&mut self, direction: Vec3) -> &mut Self {
        // update projection
        // let light_direction = math::normalize(&direction);
        // let light_position = light_direction * (self.far_plane / 2.0);
        // let light_view = math::look_at(
        //     &light_position,
        //     &math::vec3(0.0, 0.0, 0.0),
        //     &math::vec3(0.0, 1.0, 0.0),
        // );
        // self.light_space_matrix = self.shadow_projections * light_view;

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

    // /// renders the shadow map of the directional light
    // ///
    // /// # Arguments
    // /// - `models` - The models to render the shadow map for.
    // pub fn render_shadow_map(
    //     &self,
    //     drawable_nodes: &[&dyn Drawable],
    //     shadow_map: &mut DepthMapArray,
    //     index: usize,
    //     camera_world_space: &WorldTransform,
    // ) -> Vec<Mat4> {
    //     //  println!("{}", camera_postion);

    //     let vps = self.view_projection(camera_world_space);

    //     // println!("{:?}", vps);

    //     let mut depth_shader = shadow_map.prepare_shadow_map();

    //     depth_shader.bind();

    //     depth_shader.set_uniform("light.direction", self.direction);
    //     depth_shader.set_uniform("light.matrices", vps.as_slice());
    //     depth_shader.set_uniform("light.index", index as i32);
    //     depth_shader.set_uniform("light.cascadeDepth", self.num_cascades.clamp(0, 4) as i32);
    //     shadow_map.bind_framebuffer();

    //     for node in drawable_nodes {
    //         node.draw_shadow(&mut depth_shader);
    //     }

    //     shadow_map.finish_shadow_map(depth_shader);

    //     vps
    // }

    // /// bind relevent light uniforms in a shader
    // ///
    // /// does not set light space matrix
    // pub fn bind_uniforms(&mut self, shader: &mut Shader, index: usize) {
    //     shader.bind();

    //     let uniform_name = format!("directLights[{index}].direction");
    //     shader.set_uniform(&uniform_name, self.direction);
    //     let uniform_name = format!("directLights[{index}].color");
    //     shader.set_uniform(&uniform_name, self.color);
    //     let uniform_name = format!("directLights[{index}].intensity");
    //     shader.set_uniform(&uniform_name, self.intensity);
    //     let uniform_name = format!("directLights[{index}].shadowIndex");
    //     shader.set_uniform(&uniform_name, index as i32);
    //     let uniform_name = format!("directLights[{index}].cascadeLevel");
    //     shader.set_uniform(&uniform_name, self.num_cascades as i32);
    //     let uniform_name = format!("directLights[{index}].cascadeSplit");
    //     shader.set_uniform(&uniform_name, self.cascade_factors.as_slice());
    //     let uniform_name = format!("directLights[{index}].farPlane");
    //     shader.set_uniform(&uniform_name, self.far_plane);
    // }

    /// returns a buffered data for use with ssbo in shaders
    pub fn get_buffered_data(
        &self,
        shadow_index: u32,
        light_space_matrices: &[Mat4],
    ) -> DirectionalLightBufferData {
        let direction: [f32; 3] = self.direction.into();
        //// account for vec3 padding in glsl
        let sized_direction = [direction[0], direction[1], direction[2], 0.0];

        DirectionalLightBufferData {
            color: self.color.into(),
            direction: sized_direction,
            intensity: self.intensity,
            shadow_index: shadow_index as i32,
            cascade_level: self.num_cascades as i32,
            far_plane: self.far_plane,
            cascade_split: self.cascade_factors,
            light_space_matrices: Self::expand_matrix(light_space_matrices),
        }
    }

    /// expand and array of mat4 to 3d array
    fn expand_matrix(vec: &[Mat4]) -> [[[f32; 4]; 4]; 4] {
        let mut arr = [[[0.0; 4]; 4]; 4];
        let len = vec.len().min(4); // Ensure we don't exceed the array bounds

        for (i, mat) in vec.iter().take(len).enumerate() {
            for row in 0..4 {
                for col in 0..4 {
                    arr[i][row][col] = mat.row(row)[col]; // arrays are row col but linear algebra
                    // col row
                }
            }
        }

        arr
    }

    // /// binds the shadow map and light space matrix to the active shader for shaders that need shadow mapping
    // ///
    // /// # Arguments
    // /// - `shader` - The shader to bind the shadow map and light space matrix to.
    // pub fn bind_uniforms(&self, shader: &mut Shader) {
    //     let direction = math::quat_rotate_vec3(&self.transform.rotation, &math::vec3(0.0, 0.0, 1.0));
    //     // Bind shadow map and light space matrix to the active shader
    //     shader.bind();
    //     shader.set_uniform("u_lightSpaceMatrix", self.light_space_matrix);
    //     //shader.set_uniform1f("u_farShadowPlane", self.shadow_distance);
    //     shader.set_uniform("u_directLightDirection", direction);
    //     // Bind the shadow map texture to texture unit 2 (example)
    //     self.shadow_map.bind_shadow_map(shader, "shadowMap", 2);
    // }

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
            color: WHITE.into(),
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
            cascades: Vec::default(),
            num_cascades: self.num_cascades,
            direction: self.direction,
            far_plane: self.far_plane,
        };

        // create the projection for each cascade since they are independent of location
        light.gen_cascades(self.far_plane, self.num_cascades, &cascade_factors);

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
