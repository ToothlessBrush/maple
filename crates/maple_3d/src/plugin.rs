use maple_app::{Init, Plugin};
use maple_renderer::render_graph::graph::NodeLabel;

use crate::render_passes::{
    directional_shadow_pass::DirectionalShadowPass,
    main_pass::{Main, MainPass},
    point_shadow_pass::PointShadowPass,
    post_process_pass::PostProcessPass,
    shadow_resource::ShadowResource,
};

pub struct Core3D;

// Node labels for shadow passes
pub struct ShadowResourceLabel;
impl NodeLabel for ShadowResourceLabel {}

pub struct DirectionalShadowLabel;
impl NodeLabel for DirectionalShadowLabel {}

pub struct PointShadowLabel;
impl NodeLabel for PointShadowLabel {}

// Node label for post-processing
pub struct PostProcessLabel;
impl NodeLabel for PostProcessLabel {}

impl Plugin for Core3D {
    fn setup(&self, app: &mut maple_app::App<Init>) {}

    fn init(&self, app: &mut maple_app::App<maple_app::Running>) {
        let mut graph = app.renderer_mut().graph();

        // Add shadow resource management node (creates shadow textures)
        graph.add_node(ShadowResourceLabel, ShadowResource::default());

        // Add shadow passes (render depth maps)
        graph.add_node(DirectionalShadowLabel, DirectionalShadowPass::default());
        graph.add_node(PointShadowLabel, PointShadowPass::default());

        // Add main rendering pass (creates MSAA textures and renders scene)
        graph.add_node(Main, MainPass::default());

        // Add post-processing pass (blits to surface)
        graph.add_node(PostProcessLabel, PostProcessPass::default());

        // Set up execution order:
        // ShadowResource -> Shadow Passes -> Main -> PostProcess
        graph.add_edge(ShadowResourceLabel, DirectionalShadowLabel);
        graph.add_edge(ShadowResourceLabel, PointShadowLabel);
        graph.add_edge(DirectionalShadowLabel, Main);
        graph.add_edge(PointShadowLabel, Main);
        graph.add_edge(Main, PostProcessLabel);
    }
}
