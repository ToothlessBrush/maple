use wgpu::RenderPass;

pub struct CommandBuilder<'a> {
    pub(crate) backend: RenderPass<'a>,
}
