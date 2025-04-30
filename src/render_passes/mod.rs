use crate::nodes::Camera3D;
use crate::GameContext;
use crate::{nodes::node::Drawable, renderer::Renderer};

pub mod cube_shadow_pass;
pub mod main_pass;
pub mod shadow_pass;

/// represents a render pass in the renderer such as a shadow pass or geometry pass
pub trait RenderPass {
    /// functions that is called to render
    fn render(
        &self,
        renderer: &mut Renderer,
        context: &GameContext,
        drawables: &[&dyn Drawable],
        camera: &Camera3D,
    );
}
