use maple_app::{Init, Plugin};
use maple_renderer::render_graph::graph::NodeLabel;

use crate::render_passes::{
    directional_shadow_pass::DirectionalShadowPass, environment::EnvironmentPrePass,
    main_pass::MainPass, point_shadow_pass::PointShadowPass, post_process_pass::PostProcessPass,
    scene_textures::SceneTextures, shadow_resource::ShadowResource, skybox::SkyboxRender,
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

        graph.add_node(EnvironmentPrePass::default());
        graph.add_node(SceneTextures::default());
        graph.add_node(ShadowResource::default());
        graph.add_node(DirectionalShadowPass::default());
        graph.add_node(PointShadowPass::default());
        graph.add_node(SkyboxRender::default());
        graph.add_node(MainPass::default());
        graph.add_node(PostProcessPass::default());

        graph.add_edge::<EnvironmentPrePass, SkyboxRender>();
        graph.add_edge::<SceneTextures, SkyboxRender>();
        graph.add_edge::<ShadowResource, DirectionalShadowPass>();
        graph.add_edge::<ShadowResource, PointShadowPass>();
        graph.add_edge::<DirectionalShadowPass, MainPass>();
        graph.add_edge::<PointShadowPass, MainPass>();
        graph.add_edge::<SkyboxRender, MainPass>();
        graph.add_edge::<MainPass, PostProcessPass>();
    }
}
