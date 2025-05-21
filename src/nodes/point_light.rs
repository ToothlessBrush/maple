use super::Node;
use super::node::Drawable;
use super::node_builder::{Buildable, Builder, NodePrototype};
use crate::components::{EventReceiver, NodeTransform};
use crate::context::scene::Scene;
use crate::nodes::Model;
use crate::renderer::depth_cube_map_array::DepthCubeMapArray;
use crate::renderer::shader::Shader;
use crate::utils::color::WHITE;

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

    /// scene component containing its child nodes
    pub children: Scene,

    /// event receiver component
    pub events: EventReceiver,

    /// the light intensity (simply factors the color by a scale)
    intensity: f32,

    /// the light color default is White
    pub color: Vec4,

    projection: Mat4,

    //shadow_map: DepthCubeMap,
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
    pub fn new() -> PointLight {
        let transform = NodeTransform::default();

        let shadow_proj = math::perspective(math::radians(&math::vec1(90.0)).x, 1.0, 0.1, 10.0);

        PointLight {
            intensity: 1.0,
            projection: shadow_proj,
            near_plane: 0.1,
            far_plane: 10.0,
            transform,
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
        shader.set_uniform(&uniform_name, self.transform.world_space().position);
        shader.set_uniform("farPlane", self.far_plane);

        let uniform_name = format!("pointLights[{}].color", index);
        shader.set_uniform(&uniform_name, self.color);

        let uniform_name = format!("pointLights[{}].intensity", index);
        shader.set_uniform(&uniform_name, self.intensity);

        let uniform_name = format!("pointLights[{}].shadowIndex", index);
        shader.set_uniform(&uniform_name, index as i32);

        // let shadow_map_name = format!("pointLights[{}].shadowMap", index);
        // self.shadow_map
        //     .bind_shadow_map(shader, &shadow_map_name, 2 + index as u32);
    }

    pub fn get_buffered_data(&self, index: usize) -> PointLightBufferData {
        let position: [f32; 3] = self.transform.world_space().position.into();
        let sized_positon = [position[0], position[1], position[2], 0.0];

        PointLightBufferData {
            color: self.color.into(),
            position: sized_positon,
            intensity: self.intensity,
            shadow_index: index as i32,
            far_plane: self.far_plane,
            _padding: 0,
        }
    }

    /// get the nodes intensity
    pub fn get_intensity_mut(&mut self) -> &mut f32 {
        &mut self.intensity
    }

    pub fn set_intensity(&mut self, intensity: f32) {
        self.intensity = intensity;
        self.far_plane = Self::calculate_far_plane(intensity, 0.01);
        self.update_shadow_projection();
    }

    /// calculate the far_plane for a given intensity so that the shadow cutoff is at a light level
    /// thrshhold
    ///
    /// to save resources we only want to render shadows as far as the light is visible so we can
    /// cut it off at a given threshold
    pub fn calculate_far_plane(intensity: f32, threshold: f32) -> f32 {
        (intensity / threshold).sqrt()
    }

    /// this renders the shadow map from the light
    ///
    /// - `root_nodes` - a vector of the root nodes in the Scene
    /// - `world_transform` - for recursion should be default
    /// - `shadow_map` - the framebuffer to render to
    /// - `index` - lights index (should be i when you are looping through the lights)
    pub fn render_shadow_map(
        &self,
        drawable_nodes: &[&dyn Drawable],
        shadow_map: &mut DepthCubeMapArray,
        index: usize,
    ) -> [Mat4; 6] {
        let shadow_transformations = self.get_shadow_transformations();

        let depth_shader = shadow_map.prepare_shadow_map(index);
        depth_shader.bind();
        // for i in 0..6 {
        //     depth_shader.set_uniform(
        //         &format!("shadowMatrices[{}]", i),
        //         self.shadow_transformations[i],
        //     );
        // }

        println!("{:?}", self.transform);

        depth_shader.set_uniform("shadowMatrices", shadow_transformations.as_slice());
        depth_shader.set_uniform("lightPos", self.transform.world_space().position);
        depth_shader.set_uniform("farPlane", self.far_plane);
        depth_shader.set_uniform("index", index as i32);

        for node in drawable_nodes {
            node.draw_shadow(depth_shader);
        }

        shadow_map.finish_shadow_map();

        //self.last_position = camera_transform.get_position().clone();

        shadow_transformations
    }

    fn update_shadow_projection(&mut self) {
        let shadow_proj = math::perspective(
            1.0,
            math::radians(&math::vec1(90.0)).x,
            self.near_plane,
            self.far_plane,
        );

        self.projection = shadow_proj;
    }

    fn get_shadow_transformations(&self) -> [Mat4; 6] {
        let transform = self.transform.world_space();

        let shadow_proj = self.projection;

        [
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
        ]
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

impl Buildable for PointLight {
    type Builder = PointLightBuilder;
    fn builder() -> Self::Builder {
        Self::Builder {
            prototype: NodePrototype::default(),
            intensity: 1.0,
            color: WHITE.into(),
            near_plane: 0.1,
        }
    }
}

pub struct PointLightBuilder {
    prototype: NodePrototype,
    intensity: f32,
    color: Vec4,
    near_plane: f32,
}

impl Builder for PointLightBuilder {
    type Node = PointLight;
    fn prototype(&mut self) -> &mut NodePrototype {
        &mut self.prototype
    }

    fn build(&mut self) -> Self::Node {
        let proto = self.prototype().take();
        let far_plane = PointLight::calculate_far_plane(self.intensity, 0.01);
        let mut light = Self::Node {
            transform: proto.transform,
            children: proto.children,
            events: proto.events,
            color: self.color,
            intensity: self.intensity,
            near_plane: self.near_plane,
            far_plane,
            projection: Mat4::default(),
        };

        light.update_shadow_projection();
        light
    }
}

impl PointLight {
    /// set the intensity of the light
    fn intensity(&mut self, intensity: f32) -> &mut Self {
        self.intensity = intensity;
        self
    }

    /// set the color of the light
    fn color(&mut self, color: impl Into<Vec4>) -> &mut Self {
        self.color = color.into();
        self
    }

    /// near clipping plane of the light shadow projections
    fn near_plane(&mut self, near_plane: f32) -> &mut Self {
        self.near_plane = near_plane;
        self
    }
}

// /// contains point light specific build settings
// pub trait PointLightBuilder {
//     /// create a point light [NodeBuilder]
//     fn create(near_plane: f32, far_plane: f32) -> NodeBuilder<PointLight> {
//         NodeBuilder::new(PointLight::new(near_plane, far_plane))
//     }
//     /// set the color
//     fn set_color(&mut self, color: Vec4) -> &mut Self;
//
//     fn set_intensity(&mut self, intensity: f32) -> &mut Self;
// }
//
// impl PointLightBuilder for NodeBuilder<PointLight> {
//     fn set_color(&mut self, color: Vec4) -> &mut Self {
//         self.node.set_color(color);
//         self
//     }
//
//     fn set_intensity(&mut self, intensity: f32) -> &mut Self {
//         self.node.intensity = intensity;
//         self
//     }
// }
