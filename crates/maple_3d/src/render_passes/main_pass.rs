use maple::renderer::render_graph::node::RenderNode;

pub struct MainPass;

impl RenderNode for MainPass {
    fn setup(
        &mut self,
        render_ctx: &maple::renderer::core::RenderContext,
        graph_ctx: &mut maple::renderer::render_graph::graph::RenderGraphContext,
    ) -> maple::renderer::render_graph::node::RenderNodeDescriptor {
    }

    fn draw<'a>(
        &mut self,
        renderer_ctx: &maple::renderer::core::RenderContext,
        node_ctx: &mut maple::renderer::render_graph::node::RenderNodeContext,
        graph_ctx: &mut maple::renderer::render_graph::graph::RenderGraphContext,
        scene: &maple::prelude::Scene,
    ) -> Result<()> {
    }
}
