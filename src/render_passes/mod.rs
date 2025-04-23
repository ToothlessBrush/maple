pub mod main_pass;

/// represents a render pass in the renderer such as a shadow pass or geometry pass
pub trait RenderPass {
    /// functions that is called to render
    fn render(&self);
}
