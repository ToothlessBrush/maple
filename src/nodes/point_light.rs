use crate::components::NodeTransform;
use crate::context::node_manager::{Behavior, Drawable, Node, NodeManager, Ready};
use crate::context::GameContext;
use crate::nodes::Model;
use crate::renderer::depth_cube_map::DepthCubeMap;
use crate::renderer::shader::Shader;

use std::sync::{Arc, Mutex};

use crate::context::node_manager::{BehaviorCallback, ReadyCallback};

use gltf::json::extensions::root;
use nalgebra_glm::{self as glm, Mat4, Vec4};

use super::{NodeBuilder, UseBehaviorCallback, UseReadyCallback};

#[derive(Clone)]
pub struct PointLight {
    transform: NodeTransform,
    world_position: glm::Vec3, // we only want to update the projection when the light moves to avoid building it every frame
    children: NodeManager,

    /// the ready callback
    pub ready_callback: ReadyCallback<PointLight>,
    /// the behavior callback
    pub behavior_callback: BehaviorCallback<PointLight, GameContext>,
    strength: f32,

    color: Vec4,

    shadow_transformations: [Mat4; 6],

    shadow_map: DepthCubeMap,

    far_plane: f32,

    near_plane: f32,
}

impl Ready for PointLight {
    /// Calls the ready callback of the directional light.
    ///
    /// # Arguments
    /// - `self` - The directional light.
    fn ready(&mut self) {
        if let Some(callback) = self.ready_callback.take() {
            let mut guard = callback.lock().unwrap();
            guard(self);
            drop(guard);
            self.ready_callback = Some(callback)
        }
    }
}

impl Behavior for PointLight {
    /// Calls the behavior callback of the directional light.
    ///
    /// # Arguments
    /// - `self` - The directional light.
    /// - `context` - The game context.
    fn behavior(&mut self, context: &mut GameContext) {
        // take callback out of self so we can use self later
        if let Some(callback) = self.behavior_callback.take() {
            let mut guard = callback.lock().unwrap();
            guard(self, context); //"call back"
            drop(guard); // delete stupid fucking guard because its stupid and dumb
            self.behavior_callback = Some(callback);
        }
    }
}

impl Node for PointLight {
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }

    fn get_children(&mut self) -> &mut NodeManager {
        &mut self.children
    }

    fn as_ready(&mut self) -> Option<&mut (dyn Ready)> {
        Some(self)
    }

    fn as_behavior(&mut self) -> Option<&mut (dyn Behavior)> {
        Some(self)
    }
}

impl PointLight {
    pub fn new(near_plane: f32, far_plane: f32, shadow_resolution: u32) -> PointLight {
        let transform = NodeTransform::default();

        let shadow_proj =
            glm::perspective(glm::radians(&glm::vec1(90.0)).x, 1.0, near_plane, far_plane);
        let shadow_transformations = [
            shadow_proj
                * glm::look_at(
                    &transform.position,
                    &(transform.position + glm::vec3(1.0, 0.0, 0.0)),
                    &glm::vec3(0.0, -1.0, 0.0),
                ),
            shadow_proj
                * glm::look_at(
                    &transform.position,
                    &(transform.position + glm::vec3(-1.0, 0.0, 0.0)),
                    &glm::vec3(0.0, -1.0, 0.0),
                ),
            shadow_proj
                * glm::look_at(
                    &transform.position,
                    &(transform.position + glm::vec3(0.0, 1.0, 0.0)),
                    &glm::vec3(0.0, 0.0, 1.0),
                ),
            shadow_proj
                * glm::look_at(
                    &transform.position,
                    &(transform.position + glm::vec3(0.0, -1.0, 0.0)),
                    &glm::vec3(0.0, 0.0, -1.0),
                ),
            shadow_proj
                * glm::look_at(
                    &transform.position,
                    &(transform.position + glm::vec3(0.0, 0.0, 1.0)),
                    &glm::vec3(0.0, -1.0, 0.0),
                ),
            shadow_proj
                * glm::look_at(
                    &transform.position,
                    &(transform.position + glm::vec3(0.0, 0.0, -1.0)),
                    &glm::vec3(0.0, -1.0, 0.0),
                ),
        ];

        let mut shader = Shader::from_slice(
            include_str!("../../res/shaders/cubeDepthShader/cubeDepthShader.vert"),
            include_str!("../../res/shaders/cubeDepthShader/cubeDepthShader.frag"),
            Some(include_str!(
                "../../res/shaders/cubeDepthShader/cubeDepthShader.geom"
            )),
        );
        shader.bind();
        for i in 0..6 {
            shader.set_uniform(&format!("shadowMatrices[{}]", i), shadow_transformations[i]);
        }

        let shadow_map = DepthCubeMap::gen_map(shadow_resolution, shadow_resolution, shader);

        let world_position = transform.get_position().clone();

        PointLight {
            strength: 1.0,
            shadow_map,
            shadow_transformations: shadow_transformations,
            near_plane,
            far_plane,
            transform: transform,
            world_position,
            children: NodeManager::new(),
            ready_callback: None,
            behavior_callback: None,
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
        }
    }

    pub fn bind_uniforms(&mut self, shader: &mut Shader) {
        shader.bind();
        shader.set_uniform("lightPos", self.world_position);
        shader.set_uniform("farPlane", self.far_plane);
        shader.set_uniform("lightColor", self.color);

        self.shadow_map.bind_shadow_map(shader, "shadowCubeMap", 2);
    }

    pub fn render_shadow_map(
        &mut self,
        root_nodes: Vec<&mut Box<dyn Node>>,
        world_transform: NodeTransform,
    ) {
        let camera_transform = world_transform;

        //println!("{:?}", camera_transform);

        if camera_transform.position != self.world_position {
            //println!("{:?}", camera_transform);
            self.update_shadow_transformations(camera_transform);
            self.world_position = camera_transform.position.clone();
        }

        let depth_shader = self.shadow_map.prepare_shadow_map();
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

        for node in root_nodes {
            Self::draw_node_shadow(depth_shader, node, NodeTransform::default());
        }

        self.shadow_map.finish_shadow_map();

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

        for child in node.get_children() {
            Self::draw_node_shadow(shader, child.1, world_transfrom);
        }
    }

    fn update_shadow_transformations(&mut self, transform: NodeTransform) {
        // let transform = &self.transform;

        let shadow_proj = glm::perspective(
            1.0,
            glm::radians(&glm::vec1(90.0)).x,
            self.near_plane,
            self.far_plane,
        );
        let shadow_transformations = [
            shadow_proj
                * glm::look_at(
                    &transform.position,
                    &(transform.position + glm::vec3(1.0, 0.0, 0.0)),
                    &glm::vec3(0.0, -1.0, 0.0),
                ),
            shadow_proj
                * glm::look_at(
                    &transform.position,
                    &(transform.position + glm::vec3(-1.0, 0.0, 0.0)),
                    &glm::vec3(0.0, -1.0, 0.0),
                ),
            shadow_proj
                * glm::look_at(
                    &transform.position,
                    &(transform.position + glm::vec3(0.0, 1.0, 0.0)),
                    &glm::vec3(0.0, 0.0, 1.0),
                ),
            shadow_proj
                * glm::look_at(
                    &transform.position,
                    &(transform.position + glm::vec3(0.0, -1.0, 0.0)),
                    &glm::vec3(0.0, 0.0, -1.0),
                ),
            shadow_proj
                * glm::look_at(
                    &transform.position,
                    &(transform.position + glm::vec3(0.0, 0.0, 1.0)),
                    &glm::vec3(0.0, -1.0, 0.0),
                ),
            shadow_proj
                * glm::look_at(
                    &transform.position,
                    &(transform.position + glm::vec3(0.0, 0.0, -1.0)),
                    &glm::vec3(0.0, -1.0, 0.0),
                ),
        ];

        self.shadow_transformations = shadow_transformations;
    }

    pub fn set_color(&mut self, color: Vec4) -> &mut Self {
        self.color = color;
        self
    }

    /// define the ready callback of the directional light
    ///
    /// # Arguments
    /// - `ready_function` - The ready callback function of the directional light.
    pub fn define_ready<F>(&mut self, ready_function: F) -> &mut Self
    where
        F: 'static + FnMut(&mut Self) + Send + Sync,
    {
        self.ready_callback = Some(Arc::new(Mutex::new(ready_function)));
        self
    }

    /// define the behavior callback of the directional light
    ///
    /// # Arguments
    /// - `behavior_function` - The behavior callback function of the directional light.
    pub fn define_behavior<F>(&mut self, behavior_function: F) -> &mut Self
    where
        F: 'static + FnMut(&mut Self, &mut GameContext) + Send + Sync,
    {
        self.behavior_callback = Some(Arc::new(Mutex::new(behavior_function)));
        self
    }
}

pub trait PointLightBuilder {
    fn set_color(&mut self, color: Vec4) -> &mut Self;
}

impl PointLightBuilder for NodeBuilder<PointLight> {
    fn set_color(&mut self, color: Vec4) -> &mut Self {
        self.node.set_color(color);
        self
    }
}

impl UseReadyCallback for NodeBuilder<PointLight> {
    type Node = PointLight;

    fn with_ready<F>(&mut self, ready_function: F) -> &mut Self
    where
        F: 'static + FnMut(&mut Self::Node) + Send + Sync,
    {
        self.node.define_ready(ready_function);
        self
    }
}

impl UseBehaviorCallback for NodeBuilder<PointLight> {
    type Node = PointLight;

    fn with_behavior<F>(&mut self, behavior_function: F) -> &mut Self
    where
        F: 'static + FnMut(&mut Self::Node, &mut GameContext) + Send + Sync,
    {
        self.node.define_behavior(behavior_function);
        self
    }
}
