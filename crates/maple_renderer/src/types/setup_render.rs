use crate::core::Renderer;

/// this trait allows the renderer to be passes to the node which is useful if the node contains
/// any rendering data such as mesh data or other
///
/// also note the renderer doesnt automatically run this function you need to tell the engine what
/// types implement this trait with `engine.init_render::<T>()` where T implements Node and
/// RenderSetup
pub trait RenderSetup {
    fn init(&mut self, rcx: &Renderer);
}
