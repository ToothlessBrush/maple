use maple_app::{Init, Plugin};
use maple_renderer::render_graph::graph::NodeLabel;

use crate::render_passes::{
    directional_shadow_pass::DirectionalShadowPass,
    environment::{EnvironmentLabel, EnvironmentRender},
    main_pass::{Main, MainPass},
    point_shadow_pass::PointShadowPass,
    post_process_pass::PostProcessPass,
    scene_textures::{SceneTextures, SceneTexturesLabel},
    shadow_resource::ShadowResource,
    skybox::{SkyboxLabel, SkyboxRender},
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
    fn setup(&self, _app: &mut maple_app::App<Init>) {}

    fn init(&self, app: &mut maple_app::App<maple_app::Running>) {
        let mut graph = app.renderer_mut().graph();

        graph.add_node(EnvironmentLabel, EnvironmentRender::default());

        // Add scene texture resource node (creates shared render targets)
        graph.add_node(SceneTexturesLabel, SceneTextures::default());

        // Add shadow resource management node (creates shadow textures)
        graph.add_node(ShadowResourceLabel, ShadowResource::default());

        // Add shadow passes (render depth maps)
        graph.add_node(DirectionalShadowLabel, DirectionalShadowPass::default());
        graph.add_node(PointShadowLabel, PointShadowPass::default());

        // Add skybox rendering pass (renders environment cubemap as background)
        graph.add_node(SkyboxLabel, SkyboxRender::default());

        // Add main rendering pass (renders scene geometry)
        graph.add_node(Main, MainPass::default());

        // Add post-processing pass (blits to surface)
        graph.add_node(PostProcessLabel, PostProcessPass::default());

        // Set up execution order:
        // Environment -> SceneTextures -> ShadowResource -> Shadow Passes -> Skybox -> Main -> PostProcess
        graph.add_edge(EnvironmentLabel, SceneTexturesLabel);
        graph.add_edge(SceneTexturesLabel, ShadowResourceLabel);
        graph.add_edge(ShadowResourceLabel, DirectionalShadowLabel);
        graph.add_edge(ShadowResourceLabel, PointShadowLabel);
        graph.add_edge(DirectionalShadowLabel, SkyboxLabel);
        graph.add_edge(PointShadowLabel, SkyboxLabel);
        graph.add_edge(SkyboxLabel, Main);
        graph.add_edge(Main, PostProcessLabel);
    }
}
