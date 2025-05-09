use crate::nodes::DirectionalLight;

use super::RenderPass;

struct ShadowPass;

impl RenderPass for ShadowPass {
    fn render(
        &self,
        renderer: &mut crate::renderer::Renderer,
        context: &crate::context::GameContext,
        drawables: &[&dyn crate::nodes::node::Drawable],
        camera: &crate::nodes::Camera3D,
    ) {
        let lights = context.scene.collect_items::<DirectionalLight>();
    }
}
