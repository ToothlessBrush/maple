use crate::components::NodeTransform;
use crate::context::node_manager::{Behavior, Drawable, Node, NodeManager, Ready};
use crate::context::GameContext;
use crate::nodes::Model;
use crate::renderer::depth_cube_map::DepthCubeMap;
use crate::renderer::shader::Shader;

use nalgebra_glm::{self as glm, Mat4};

pub struct PointLight {
    transform: NodeTransform,
    last_position: glm::Vec3, // we only want to update the projection when the light moves to avoid building it every frame
    children: NodeManager,
    /// The ready callback of the directional light.
    ready_callback: Option<Box<dyn FnMut(&mut Self)>>,
    /// The behavior callback of the directional light.
    behavior_callback: Option<Box<dyn FnMut(&mut Self, &mut GameContext)>>,

    strength: u32,

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
        if let Some(mut callback) = self.ready_callback.take() {
            callback(self);
            self.ready_callback = Some(callback);
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
        if let Some(mut callback) = self.behavior_callback.take() {
            callback(self, context);
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
    pub fn new(
        transform: NodeTransform,
        strength: u32,
        near_plane: f32,
        far_plane: f32,
        shadow_resoultion: u32,
    ) -> PointLight {
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

        let shadow_map = DepthCubeMap::gen_map(shadow_resoultion, shadow_resoultion, shader);

        let last_position = transform.get_position().clone();

        PointLight {
            strength,
            shadow_map,
            shadow_transformations: shadow_transformations,
            near_plane,
            far_plane,
            transform: transform,
            last_position,
            children: NodeManager::new(),
            ready_callback: None,
            behavior_callback: None,
        }
    }

    pub fn bind_uniforms(&mut self, shader: &mut Shader) {
        shader.bind();
        shader.set_uniform("lightPos", *self.transform.get_position());
        shader.set_uniform("farPlane", self.far_plane);

        self.shadow_map.bind_shadow_map(shader, "shadowCubeMap", 2);
    }

    pub fn render_shadow_map(&mut self, models: &mut [&mut Model]) {
        if *self.transform.get_position() != self.last_position {
            self.update_shadow_transformations();
        }

        self.shadow_map.render_shadow_map(&mut |depth_shader| {
            depth_shader.bind();
            // for i in 0..6 {
            //     depth_shader.set_uniform(
            //         &format!("shadowMatrices[{}]", i),
            //         self.shadow_transformations[i],
            //     );
            // }
            depth_shader.set_uniform("shadowMatrices", self.shadow_transformations.as_slice());
            depth_shader.set_uniform("lightPos", *self.transform.get_position());
            depth_shader.set_uniform("farPlane", self.far_plane);

            for model in models.iter_mut() {
                model.draw_shadow(depth_shader);
            }
            depth_shader.unbind();
        });

        self.last_position = self.transform.get_position().clone();
    }

    fn update_shadow_transformations(&mut self) {
        let transform = &self.transform;

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

    /// define the ready callback of the directional light
    ///
    /// # Arguments
    /// - `ready_function` - The ready callback function of the directional light.
    pub fn define_ready<F>(&mut self, ready_function: F) -> &mut Self
    where
        F: 'static + FnMut(&mut Self),
    {
        self.ready_callback = Some(Box::new(ready_function));
        self
    }

    /// define the behavior callback of the directional light
    ///
    /// # Arguments
    /// - `behavior_function` - The behavior callback function of the directional light.
    pub fn define_behavior<F>(&mut self, behavior_function: F) -> &mut Self
    where
        F: 'static + FnMut(&mut Self, &mut GameContext),
    {
        self.behavior_callback = Some(Box::new(behavior_function));
        self
    }
}
