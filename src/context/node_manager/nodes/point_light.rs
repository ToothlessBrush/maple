use crate::context::node_manager::nodes::Model;
use crate::context::node_manager::{Behavior, Drawable, Node, NodeManager, NodeTransform, Ready};
use crate::context::GameContext;
use crate::renderer::depth_cube_map::DepthCubeMap;
use crate::renderer::shader::Shader;

use nalgebra_glm::{self as glm, Mat4};

struct PointLight {
    transform: NodeTransform,
    children: NodeManager,
    /// The ready callback of the directional light.
    ready_callback: Option<Box<dyn FnMut(&mut Self)>>,
    /// The behavior callback of the directional light.
    behavior_callback: Option<Box<dyn FnMut(&mut Self, &mut GameContext)>>,

    strength: u32,

    shadow_transformations: [Mat4; 6],

    shadow_map: DepthCubeMap,
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
    fn new(
        transform: NodeTransform,
        strength: u32,
        far_plane: f32,
        shadow_resoultion: u32,
    ) -> PointLight {
        let shadow_proj = glm::perspective(glm::radians(&glm::vec1(90.0)).x, 1.0, 0.1, far_plane);
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
            include_str!("../../../../res/shaders/cubeDepthShader/cubeDepthShader.vert"),
            include_str!("../../../../res/shaders/cubeDepthShader/cubeDepthShader.frag"),
            Some(include_str!(
                "../../../../res/shaders/cubeDepthShader/cubeDepthShader.geom"
            )),
        );
        shader.bind();
        for i in 0..6 {
            shader.set_uniform(&format!("shadowMatrices[{}]", i), shadow_transformations[i]);
        }

        let shadow_map = DepthCubeMap::gen_map(shadow_resoultion, shadow_resoultion, shader);

        PointLight {
            strength,
            shadow_map,
            shadow_transformations: shadow_transformations,
            transform: transform,
            children: NodeManager::new(),
            ready_callback: None,
            behavior_callback: None,
        }
    }

    pub fn render_shadow_map(&mut self, models: &mut [&mut Model]) {
        self.shadow_map.render_shadow_map(&mut |depth_shader| {
            depth_shader.bind();
            for i in 0..6 {
                depth_shader.set_uniform(
                    &format!("shadowMatrices[{}]", i),
                    self.shadow_transformations[i],
                );
            }
            for model in models.iter_mut() {
                model.draw_shadow(depth_shader);
            }
            depth_shader.unbind();
        });
    }
}
