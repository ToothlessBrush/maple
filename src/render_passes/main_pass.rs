use super::RenderPass;
use crate::nodes::node::Drawable;
use crate::nodes::Camera3D;
use crate::renderer::Renderer;
use crate::GameContext;

/// the main pass is what is rendered to the scene (before any post processing)
pub struct MainPass;

impl RenderPass for MainPass {
    fn render(
        &self,
        renderer: &mut Renderer,
        _context: &GameContext,
        drawables: &[&dyn Drawable],
        camera: &Camera3D,
    ) {
        for item in drawables {
            item.draw(&mut renderer.default_shader, camera);
        }
    }
}
