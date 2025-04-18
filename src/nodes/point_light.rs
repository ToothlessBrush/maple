use super::node::Drawable;
use super::Node;
use crate::components::{EventReceiver, NodeTransform};
use crate::context::scene::Scene;
use crate::nodes::Model;
use crate::renderer::depth_cube_map_array::DepthCubeMapArray;
use crate::renderer::shader::Shader;

use nalgebra_glm::{self as math, Mat4, Vec4};

use super::NodeBuilder;

/// used to pass data to the shader buffer
///
/// the data on the gpu follows this format in this order:
/// ```c
/// struct PointLight {
///     vec4 color;
///     vec4 pos;
///     float intensity;
///     int shadowIndex;
///     float far_plane;
///     int _padding;
/// };
/// ```
#[repr(C)]
#[derive(Debug, Clone)]
pub struct PointLightBufferData {
    color: [f32; 4],
    position: [f32; 4],
    intensity: f32,
    shadow_index: i32,
    far_plane: f32,
    _padding: i32, //ssbo is 16 byte aligned
}

/// point lights nodes represent point lights in the Scene
///
/// point lights are lights that are cast from a single point. light is calculated by getting the
/// distance and direction to the point light position.
#[derive(Clone)]
pub struct PointLight {
    /// transform component for point light
    pub transform: NodeTransform,

    world_position: math::Vec3,

    /// scene component containing its child nodes
    pub children: Scene,

    /// event receiver component
    pub events: EventReceiver,

    /// the light intensity (simply factors the color by a scale)
    pub intensity: f32,

    /// the light color default is White
    pub color: Vec4,

    shadow_transformations: [Mat4; 6],

    //shadow_map: DepthCubeMap,
    shadow_map_index: usize,

    far_plane: f32,

    near_plane: f32,
}

impl Node for PointLight {
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

impl PointLight {
    /// create a point light you should use [NodeBuilder] if you are cool.
    pub fn new(near_plane: f32, far_plane: f32) -> PointLight {
        let transform = NodeTransform::default();

        let shadow_proj = math::perspective(
            math::radians(&math::vec1(90.0)).x,
            1.0,
            near_plane,
            far_plane,
        );
        let shadow_transformations = [
            shadow_proj
                * math::look_at(
                    &transform.position,
                    &(transform.position + math::vec3(1.0, 0.0, 0.0)),
                    &math::vec3(0.0, -1.0, 0.0),
                ),
            shadow_proj
                * math::look_at(
                    &transform.position,
                    &(transform.position + math::vec3(-1.0, 0.0, 0.0)),
                    &math::vec3(0.0, -1.0, 0.0),
                ),
            shadow_proj
                * math::look_at(
                    &transform.position,
                    &(transform.position + math::vec3(0.0, 1.0, 0.0)),
                    &math::vec3(0.0, 0.0, 1.0),
                ),
            shadow_proj
                * math::look_at(
                    &transform.position,
                    &(transform.position + math::vec3(0.0, -1.0, 0.0)),
                    &math::vec3(0.0, 0.0, -1.0),
                ),
            shadow_proj
                * math::look_at(
                    &transform.position,
                    &(transform.position + math::vec3(0.0, 0.0, 1.0)),
                    &math::vec3(0.0, -1.0, 0.0),
                ),
            shadow_proj
                * math::look_at(
                    &transform.position,
                    &(transform.position + math::vec3(0.0, 0.0, -1.0)),
                    &math::vec3(0.0, -1.0, 0.0),
                ),
        ];

        // let mut shader = Shader::from_slice(
        //     include_str!("../../res/shaders/cubeDepthShader/cubeDepthShader.vert"),
        //     include_str!("../../res/shaders/cubeDepthShader/cubeDepthShader.frag"),
        //     Some(include_str!(
        //         "../../res/shaders/cubeDepthShader/cubeDepthShader.geom"
        //     )),
        // );
        // shader.bind();
        // for i in 0..6 {
        //     shader.set_uniform(&format!("shadowMatrices[{}]", i), shadow_transformations[i]);
        // }

        // let shadow_map = DepthCubeMap::gen_map(shadow_resolution, shadow_resolution, shader);

        let world_position = *transform.get_position();

        PointLight {
            intensity: 1.0,
            // shadow_map,
            shadow_map_index: 0,
            shadow_transformations,
            near_plane,
            far_plane,
            transform,
            world_position,
            children: Scene::new(),
            events: EventReceiver::new(),
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
        }
    }

    /// bind related uniforms if lights are passed to the shader via uniforms
    ///
    /// this sets pointLights[i].pos, .color, .intensity, .shadowIndex
    pub fn bind_uniforms(&mut self, shader: &mut Shader, index: usize) {
        shader.bind();

        let uniform_name = format!("pointLights[{}].pos", index);
        shader.set_uniform(&uniform_name, self.world_position);
        shader.set_uniform("farPlane", self.far_plane);

        let uniform_name = format!("pointLights[{}].color", index);
        shader.set_uniform(&uniform_name, self.color);

        let uniform_name = format!("pointLights[{}].intensity", index);
        shader.set_uniform(&uniform_name, self.intensity);

        let uniform_name = format!("pointLights[{}].shadowIndex", index);
        shader.set_uniform(&uniform_name, self.shadow_map_index as i32);

        // let shadow_map_name = format!("pointLights[{}].shadowMap", index);
        // self.shadow_map
        //     .bind_shadow_map(shader, &shadow_map_name, 2 + index as u32);
    }

    pub fn get_buffered_data(&self) -> PointLightBufferData {
        let position: [f32; 3] = self.world_position.into();
        let sized_positon = [position[0], position[1], position[2], 0.0];

        PointLightBufferData {
            color: self.color.into(),
            position: sized_positon,
            intensity: self.intensity,
            shadow_index: self.shadow_map_index as i32,
            far_plane: self.far_plane,
            _padding: 0,
        }
    }

    /// get the nodes intensity
    pub fn get_intensity_mut(&mut self) -> &mut f32 {
        &mut self.intensity
    }

    /// this renders the shadow map from the light
    ///
    /// - `root_nodes` - a vector of the root nodes in the Scene
    /// - `world_transform` - for recursion should be default
    /// - `shadow_map` - the framebuffer to render to
    /// - `index` - lights index (should be i when you are looping through the lights)
    pub fn render_shadow_map(
        &mut self,
        root_nodes: Vec<&mut Box<dyn Node>>,
        world_transform: NodeTransform,
        shadow_map: &mut DepthCubeMapArray,
        index: usize,
    ) {
        self.shadow_map_index = index;
        let camera_transform = world_transform;

        //println!("{:?}", camera_transform);

        if camera_transform.position != self.world_position {
            //println!("{:?}", camera_transform);
            self.update_shadow_transformations(camera_transform);
            self.world_position = camera_transform.position;
        }

        let depth_shader = shadow_map.prepare_shadow_map(self.shadow_map_index);
        depth_shader.bind();
        // for i in 0..6 {
        //     depth_shader.set_uniform(
        //         &format!("shadowMatrices[{}]", i),
        //         self.shadow_transformations[i],
        //     );
        // }
        depth_shader.set_uniform("shadowMatrices", self.shadow_transformations.as_slice());
        depth_shader.set_uniform("lightPos", self.world_position);
        depth_shader.set_uniform("farPlane", self.far_plane);
        depth_shader.set_uniform("index", self.shadow_map_index as i32);

        for node in root_nodes {
            Self::draw_node_shadow(depth_shader, node, NodeTransform::default());
        }

        shadow_map.finish_shadow_map();

        //self.last_position = camera_transform.get_position().clone();
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

    fn update_shadow_transformations(&mut self, transform: NodeTransform) {
        // let transform = &self.transform;

        let shadow_proj = math::perspective(
            1.0,
            math::radians(&math::vec1(90.0)).x,
            self.near_plane,
            self.far_plane,
        );
        let shadow_transformations = [
            shadow_proj
                * math::look_at(
                    &transform.position,
                    &(transform.position + math::vec3(1.0, 0.0, 0.0)),
                    &math::vec3(0.0, -1.0, 0.0),
                ),
            shadow_proj
                * math::look_at(
                    &transform.position,
                    &(transform.position + math::vec3(-1.0, 0.0, 0.0)),
                    &math::vec3(0.0, -1.0, 0.0),
                ),
            shadow_proj
                * math::look_at(
                    &transform.position,
                    &(transform.position + math::vec3(0.0, 1.0, 0.0)),
                    &math::vec3(0.0, 0.0, 1.0),
                ),
            shadow_proj
                * math::look_at(
                    &transform.position,
                    &(transform.position + math::vec3(0.0, -1.0, 0.0)),
                    &math::vec3(0.0, 0.0, -1.0),
                ),
            shadow_proj
                * math::look_at(
                    &transform.position,
                    &(transform.position + math::vec3(0.0, 0.0, 1.0)),
                    &math::vec3(0.0, -1.0, 0.0),
                ),
            shadow_proj
                * math::look_at(
                    &transform.position,
                    &(transform.position + math::vec3(0.0, 0.0, -1.0)),
                    &math::vec3(0.0, -1.0, 0.0),
                ),
        ];

        self.shadow_transformations = shadow_transformations;
    }

    /// set the light color
    pub fn set_color(&mut self, color: impl Into<Vec4>) -> &mut Self {
        let color = color.into();
        self.color = color;
        self
    }

    /// get the light color
    pub fn get_color_mut(&mut self) -> &mut Vec4 {
        &mut self.color
    }
}

/// contains point light specific build settings
pub trait PointLightBuilder {
    /// create a point light [NodeBuilder]
    fn create(near_plane: f32, far_plane: f32) -> NodeBuilder<PointLight> {
        NodeBuilder::new(PointLight::new(near_plane, far_plane))
    }
    /// set the color
    fn set_color(&mut self, color: Vec4) -> &mut Self;
}

impl PointLightBuilder for NodeBuilder<PointLight> {
    fn set_color(&mut self, color: Vec4) -> &mut Self {
        self.node.set_color(color);
        self
    }
}
