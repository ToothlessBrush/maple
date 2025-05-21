use super::RenderPass;
use crate::GameContext;
use crate::nodes::Camera3D;
use crate::nodes::node::Drawable;
use crate::renderer::Renderer;

/// the main pass is what is rendered to the scene (before any post processing)
pub struct MainPass;

impl RenderPass for MainPass {
    fn render(
        &self,
        renderer: &mut Renderer,
        context: &GameContext,
        drawables: &[&dyn Drawable],
        camera: &Camera3D,
    ) {
        Renderer::viewport(
            context.window.get_framebuffer_size().0,
            context.window.get_framebuffer_size().1,
        );

        renderer.default_shader.bind();

        renderer.direct_light_buffer.bind(0);
        renderer.point_light_buffer.bind(1);

        renderer
            .default_shader
            .set_uniform("scene.biasFactor", renderer.scene_state.bias_factor);
        renderer
            .default_shader
            .set_uniform("scene.biasOffset", renderer.scene_state.bias_offset);
        renderer
            .default_shader
            .set_uniform("scene.ambient", renderer.scene_state.ambient_light);

        let (window_width, window_height) = context.window.get_size();

        renderer.default_shader.set_uniform(
            "u_VP",
            camera.get_vp_matrix(window_width as f32 / window_height as f32),
        );

        for item in drawables {
            item.draw(&mut renderer.default_shader, camera);
        }
    }
}
