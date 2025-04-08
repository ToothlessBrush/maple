//! Directional light casts light on a scene from a single direction, like the sun. It is used to simulate sunlight in a scene. It is a type of light that is infinitely far away and has no attenuation. It is defined by a direction and a color. It can also cast shadows using a shadow map.
//!
//! ## Usage
//! add this to the node tree to add a directional light to the scene.
use super::node::Drawable;
use super::Node;
use crate::components::{EventReceiver, NodeTransform};
use crate::context::scene::Scene;
use crate::nodes::Model;
use crate::renderer::depth_map_array::DepthMapArray;
use crate::renderer::shader::Shader;
use crate::utils::color::Color;
use nalgebra_glm::{self as math, Mat4};

use super::NodeBuilder;

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
///     float cascadeSplit[4];
///     mat4 lightSpaceMatrices[4];
///     float farPlane;
/// };
/// ```
#[repr(C)]
#[derive(Clone, Debug)]
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

/// Directional light casts light on a scene from a single direction, like the sun. It is used to simulate sunlight in a scene. It is a type of light that is infinitely far away and has no attenuation. It is defined by a direction and a color. It can also cast shadows using a shadow map.
///
/// ## Usage
/// add this to the node tree to add a directional light to the scene.
#[derive(Clone)]
pub struct DirectionalLight {
    /// The transform of the directional light.
    transform: NodeTransform,
    /// The children of the directional light.
    children: Scene,

    events: EventReceiver,
    /// The color of the directional light.
    pub color: math::Vec4,
    /// The intensity of the directional light.
    pub intensity: f32,
    /// The projection matrix of the shadow cast by the directional light.
    shadow_projections: math::Mat4,
    /// The light space matrix of the shadow cast by the directional light.
    light_space_matrices: Vec<math::Mat4>,
    /// direction to the light
    pub direction: math::Vec3,

    far_plane: f32,

    shadow_index: usize,

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

    fn get_children_mut(&mut self) -> &mut crate::context::scene::Scene {
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
        direction: math::Vec3,
        color: impl Into<math::Vec4>,
        shadow_distance: f32,
        num_cascades: usize,
        //cascade_factors: &[f32],
    ) -> DirectionalLight {
        let shadow_projections = math::ortho(
            shadow_distance / 2.0,
            shadow_distance / 2.0,
            shadow_distance / 2.0,
            shadow_distance / 2.0,
            0.1,
            shadow_distance,
        );

        //let direction = math::vec3(0.0, 0.0, 1.0);
        // let light_direction = math::normalize(&direction);
        // let light_position = light_direction * (shadow_distance / 2.0);
        // calculate the rotation quaternion from the orientation vector
        //let direction = math::normalize(&direction);
        let reference = math::vec3(0.0, 0.0, 1.0);

        // Handle parallel and anti-parallel cases
        let rotation_quat = if math::dot(&direction, &reference).abs() > 0.9999 {
            if direction.z > 0.0 {
                math::quat_identity() // No rotation needed
            } else {
                math::quat_angle_axis(math::pi::<f32>(), &math::vec3(1.0, 0.0, 0.0))
                // 180-degree rotation
            }
        } else {
            let rotation_axis = math::cross(&reference, &direction).normalize();
            let rotation_angle = math::dot(&reference, &direction).acos();
            math::quat_angle_axis(rotation_angle, &rotation_axis)
        };

        // println!("Directional Light Rotation: {:?}", rotation_quat);
        // println!("Directional Light Direction: {:?}", direction);

        // let check_direction = math::quat_rotate_vec3(&rotation_quat, &reference);
        // println!("Directional Light Check Direction: {:?}", check_direction);

        // Use a tolerance-based assertion for floating-point comparisons
        // assert!((check_direction - direction).magnitude() < 1e-5);

        let cascade_factors =
            Self::calculate_cascade_splits(0.1, shadow_distance, num_cascades, 0.7);

        println!("{:?}", cascade_factors);

        let mut light = DirectionalLight {
            transform: NodeTransform::new(
                math::vec3(0.0, 0.0, 0.0),
                rotation_quat,
                math::vec3(1.0, 1.0, 1.0),
            ),
            children: Scene::new(),
            events: EventReceiver::new(),
            intensity: 1.0,
            color: color.into(),
            shadow_projections,
            light_space_matrices: Vec::new(),
            shadow_index: 0,
            cascades: Vec::default(),
            num_cascades,
            direction: math::normalize(&direction),
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

            let projection = math::ortho(-radius, radius, -radius, radius, near_plane, far_plane);

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
    /// - 'camera_pos' - world position of the camera (since shadow projections are centered around camera)
    ///
    /// # Returns
    /// the view_projection matrix
    pub fn get_vps(&self, camera_pos: &math::Vec3) -> Vec<Mat4> {
        let projection_offset = math::normalize(&self.direction) * (self.far_plane / 2.0);
        let view = math::look_at(
            &(camera_pos + projection_offset),
            camera_pos,
            &math::vec3(0.0, 1.0, 0.0),
        );

        // println!("{:?}", view);

        // projection matrix doesnt change so we can just combine them to get the set of vp matrices
        let vps = self
            .cascades
            .iter()
            .map(|cascade| cascade.projection * view)
            .collect();

        vps
    }

    /// direction the lights coming from
    pub fn set_direction(&mut self, direction: math::Vec3) -> &mut Self {
        // update projection
        // let light_direction = math::normalize(&direction);
        // let light_position = light_direction * (self.far_plane / 2.0);
        // let light_view = math::look_at(
        //     &light_position,
        //     &math::vec3(0.0, 0.0, 0.0),
        //     &math::vec3(0.0, 1.0, 0.0),
        // );
        // self.light_space_matrix = self.shadow_projections * light_view;

        let reference = math::vec3(0.0, 0.0, 1.0);

        // update rotation
        let rotation_quat = if math::dot(&direction, &reference).abs() > 0.9999 {
            if direction.z > 0.0 {
                math::quat_identity() // No rotation needed
            } else {
                math::quat_angle_axis(math::pi::<f32>(), &math::vec3(1.0, 0.0, 0.0))
                // 180-degree rotation
            }
        } else {
            let rotation_axis = math::cross(&reference, &direction).normalize();
            let rotation_angle = math::dot(&reference, &direction).acos();
            math::quat_angle_axis(rotation_angle, &rotation_axis)
        };

        self.transform.set_rotation(rotation_quat);

        self
    }

    /// sets the color of the light
    pub fn set_color(&mut self, color: Color) -> &mut Self {
        self.color = color.into();
        self
    }

    /// set the intensity of the light
    pub fn set_intensity(&mut self, intensity: f32) -> &mut Self {
        self.intensity = intensity;
        self
    }
    /// renders the shadow map of the directional light
    ///
    /// # Arguments
    /// - `models` - The models to render the shadow map for.
    pub fn render_shadow_map(
        &mut self,
        root_nodes: Vec<&mut Box<dyn Node>>,
        shadow_map: &mut DepthMapArray,
        index: usize,
        camera_world_space: &NodeTransform,
    ) {
        self.shadow_index = index;
        let camera_postion = camera_world_space.position;

        //  println!("{}", camera_postion);

        let vps = self.get_vps(&camera_postion);

        // println!("{:?}", vps);

        let mut depth_shader = shadow_map.prepare_shadow_map();

        depth_shader.bind();

        depth_shader.set_uniform("light.direction", self.direction);
        depth_shader.set_uniform("light.matrices", vps.as_slice());
        depth_shader.set_uniform("light.index", index as i32);
        depth_shader.set_uniform("light.cascadeDepth", self.num_cascades.clamp(0, 4) as i32);
        shadow_map.bind();

        self.light_space_matrices = vps;

        for node in root_nodes {
            Self::draw_node_shadow(&mut depth_shader, node, NodeTransform::default());
        }

        shadow_map.finish_shadow_map(depth_shader);
    }

    /// bind relevent light uniforms in a shader
    pub fn bind_uniforms(&mut self, shader: &mut Shader, index: usize) {
        shader.bind();

        let uniform_name = format!("directLights[{index}].direction");
        shader.set_uniform(&uniform_name, self.direction);
        let uniform_name = format!("directLights[{index}].color");
        shader.set_uniform(&uniform_name, self.color);
        let uniform_name = format!("directLights[{index}].intensity");
        shader.set_uniform(&uniform_name, self.intensity);
        let uniform_name = format!("directLights[{index}].shadowIndex");
        shader.set_uniform(&uniform_name, self.shadow_index as i32);
        let uniform_name = format!("directLights[{index}].cascadeLevel");
        shader.set_uniform(&uniform_name, self.num_cascades as i32);
        let uniform_name = format!("directLights[{index}].cascadeSplit");
        shader.set_uniform(&uniform_name, self.cascade_factors.as_slice());
        let uniform_name = format!("directLights[{index}].farPlane");
        shader.set_uniform(&uniform_name, self.far_plane);
        let uniform_name = format!("directLights[{index}].lightSpaceMatrices");
        shader.set_uniform(&uniform_name, self.light_space_matrices.as_slice());
    }

    /// returns a buffered data for use with ssbo in shaders
    pub fn get_buffered_data(&self) -> DirectionalLightBufferData {
        let direction: [f32; 3] = self.direction.into();
        //// account for vec3 padding in glsl
        let sized_direction = [direction[0], direction[1], direction[2], 0.0];

        DirectionalLightBufferData {
            color: self.color.into(),
            direction: sized_direction,
            intensity: self.intensity,
            shadow_index: self.shadow_index as i32,
            cascade_level: self.num_cascades as i32,
            far_plane: self.far_plane,
            cascade_split: self.cascade_factors,
            light_space_matrices: Self::expand_matrix(&self.light_space_matrices),
        }
    }

    /// expand and array of mat4 to 3d array
    fn expand_matrix(vec: &[Mat4]) -> [[[f32; 4]; 4]; 4] {
        let mut arr = [[[0.0; 4]; 4]; 4];
        let len = vec.len().min(4); // Ensure we don't exceed the array bounds

        for (i, mat) in vec.iter().take(len).enumerate() {
            for row in 0..4 {
                for col in 0..4 {
                    arr[i][row][col] = mat[(col, row)]; // arrays are row col but linear algebra
                                                        // col row
                }
            }
        }

        arr
    }

    fn draw_node_shadow(
        shader: &mut Shader,
        node: &mut Box<dyn Node>,
        parent_transform: NodeTransform,
    ) {
        let world_transfrom = parent_transform + *node.get_transform();
        if let Some(model) = node.as_any_mut().downcast_mut::<Model>() {
            model.draw_shadow(shader, world_transfrom);
        }

        for child in node.get_children_mut() {
            Self::draw_node_shadow(shader, child.1, world_transfrom);
        }
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
        self.shadow_projections = math::ortho(
            -self.far_plane / 2.0,
            self.far_plane / 2.0,
            -self.far_plane / 2.0,
            self.far_plane / 2.0,
            0.1,
            self.far_plane,
        );
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

/// [DirectionalLight] specific build methods for [NodeBuilder]
pub trait DirectionalLightBuilder {
    /// create a NodeBuilder for Directional light
    ///
    /// # Arguements
    /// - `direction` - direction is a vec3 that points towards the source
    /// - `color` - color of the light
    ///
    /// # returns
    /// a DirectionalLight NodeBuilder
    fn create(direction: math::Vec3, color: math::Vec4) -> NodeBuilder<DirectionalLight> {
        NodeBuilder::new(DirectionalLight::new(direction, color, 1000.0, 4))
    }

    /// set the direction of the light it points towards the source
    fn set_direction(&mut self, direction: math::Vec3) -> &mut Self;
    /// set the intensity of the light. default: 1.0
    fn set_intensity(&mut self, intensity: f32) -> &mut Self;
    /// set the color of the light
    fn set_color(&mut self, color: Color) -> &mut Self;
    /// how far shadows will be rendered during the shadow pass
    fn set_far_plane(&mut self, far: f32) -> &mut Self;
}

impl DirectionalLightBuilder for NodeBuilder<DirectionalLight> {
    fn set_direction(&mut self, direction: nalgebra_glm::Vec3) -> &mut Self {
        self.node.set_direction(direction);
        self
    }
    fn set_color(&mut self, color: Color) -> &mut Self {
        self.node.set_color(color);
        self
    }
    fn set_intensity(&mut self, intensity: f32) -> &mut Self {
        self.node.set_intensity(intensity);
        self
    }
    fn set_far_plane(&mut self, far: f32) -> &mut Self {
        self.node.set_far_plane(far);
        self
    }
}
