use std::path::Iter;

use crate::engine::game_context::node_manager::{Drawable, Node};
use crate::engine::game_context::nodes::model::Model;
use crate::engine::game_context::GameContext;
use crate::engine::renderer::shader::Shader;
use crate::engine::renderer::shadow_map::ShadowMap;
use nalgebra_glm as glm;

pub struct DirectionalLightTransform {
    pub direction: glm::Vec3,
}

pub struct DirectionalLight {
    transform: DirectionalLightTransform,
    pub color: glm::Vec3,
    pub intensity: f32,
    shadow_distance: f32,
    shadow_projections: glm::Mat4,
    light_space_matrix: glm::Mat4,

    shadow_map: ShadowMap,

    ready_callback: Option<Box<dyn FnMut(&mut Self)>>,
    behavior_callback: Option<Box<dyn FnMut(&mut Self, &mut GameContext)>>,
}

impl Node for DirectionalLight {
    type Transform = DirectionalLightTransform;

    fn get_model_matrix(&self) -> glm::Mat4 {
        glm::identity()
    }

    fn get_transform(&self) -> &Self::Transform {
        &self.transform
    }

    fn define_ready<F>(&mut self, ready_function: F) -> &mut Self
    where
        F: 'static + FnMut(&mut Self),
    {
        self.ready_callback = Some(Box::new(ready_function));
        self
    }

    fn define_behavior<F>(&mut self, behavior_function: F) -> &mut Self
    where
        F: 'static + FnMut(&mut Self, &mut GameContext),
    {
        self.behavior_callback = Some(Box::new(behavior_function));
        self
    }

    //if the model has a ready function then call it
    fn ready(&mut self) {
        if let Some(mut callback) = self.ready_callback.take() {
            callback(self);
            self.ready_callback = Some(callback);
        }
    }

    //if the model has a behavior function then call it
    fn behavior(&mut self, context: &mut GameContext) {
        if let Some(mut callback) = self.behavior_callback.take() {
            callback(self, context);
            self.behavior_callback = Some(callback);
        }
    }
}

impl DirectionalLight {
    pub fn new(
        direction: glm::Vec3,
        color: glm::Vec3,
        intensity: f32,
        shadow_distance: f32,
        shadow_resolution: u32,
    ) -> DirectionalLight {
        let shadow_projections = glm::ortho(
            -shadow_distance / 2.0,
            shadow_distance / 2.0,
            -shadow_distance / 2.0,
            shadow_distance / 2.0,
            0.1,
            shadow_distance,
        );
        let light_direction = glm::normalize(&direction);
        let light_position = light_direction * (shadow_distance / 2.0);
        let light_view = glm::look_at(
            &light_position,
            &glm::vec3(0.0, 0.0, 0.0),
            &glm::vec3(0.0, 1.0, 0.0),
        );
        let light_space_matrix = shadow_projections * light_view;

        let shadow_shader = Shader::new(
            "res/shaders/depthShader/depthShader.vert",
            "res/shaders/depthShader/depthShader.frag",
            None,
        );

        let shadow_map = ShadowMap::gen_map(
            shadow_resolution as i32,
            shadow_resolution as i32,
            shadow_shader,
        );

        DirectionalLight {
            transform: DirectionalLightTransform { direction },
            color,
            intensity,
            shadow_distance,
            shadow_projections,
            light_space_matrix,
            shadow_map,
            ready_callback: None,
            behavior_callback: None,
        }
    }

    pub fn render_shadow_map(&mut self, models: &mut dyn std::iter::Iterator<Item = &mut Model>) {
        self.shadow_map.render_shadow_map(&mut |depth_shader| {
            depth_shader.bind();
            for model in models.into_iter() {
                model.draw_shadow(depth_shader, &self.light_space_matrix);
            }
            depth_shader.unbind();
        });
    }

    /// binds the shadow map and light space matrix to the active shader for shaders that need shadow mapping
    pub fn bind_uniforms(&self, shader: &mut Shader) {
        // Bind shadow map and light space matrix to the active shader
        shader.bind();
        shader.set_uniform_mat4f("u_lightSpaceMatrix", &self.light_space_matrix);
        shader.set_uniform1f("u_farShadowPlane", self.shadow_distance);
        shader.set_uniform3f(
            "u_directLightDirection",
            self.transform.direction.x,
            self.transform.direction.y,
            self.transform.direction.z,
        );
        // Bind the shadow map texture to texture unit 2 (example)
        self.shadow_map.bind_shadow_map(shader, "shadowMap", 2);
    }

    pub fn get_far_plane(&self) -> f32 {
        self.shadow_distance
    }

    pub fn set_far_plane(&mut self, distance: f32) {
        self.shadow_distance = distance;
        self.shadow_projections = glm::ortho(
            -self.shadow_distance / 2.0,
            self.shadow_distance / 2.0,
            -self.shadow_distance / 2.0,
            self.shadow_distance / 2.0,
            0.1,
            self.shadow_distance,
        );
        let light_direction = glm::normalize(&self.transform.direction);
        let light_position = light_direction * (self.shadow_distance / 2.0); //self.shadow_distance;
        let light_view = glm::look_at(
            &light_position,
            &glm::vec3(0.0, 0.0, 0.0),
            &glm::vec3(0.0, 1.0, 0.0),
        );
        self.light_space_matrix = self.shadow_projections * light_view;
    }
}
