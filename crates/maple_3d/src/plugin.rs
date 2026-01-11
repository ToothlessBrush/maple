use maple_app::{Init, Plugin};

use crate::render_passes::{
    directional_shadow_pass::DirectionalShadowPass, environment::EnvironmentPrePass,
    main_pass::MainPass, point_shadow_pass::PointShadowPass, post_process_pass::CompositePass,
    scene_textures::SceneTextures, shadow_resource::ShadowResource, skybox::SkyboxRender,
};

pub struct Core3D;

impl Plugin for Core3D {
    fn setup(&self, _app: &mut maple_app::App<Init>) {}

    fn init(&self, app: &mut maple_app::App<maple_app::Running>) {
        let mut graph = app.renderer_mut().graph();

        graph.add_node_with(EnvironmentPrePass::setup);
        graph.add_node_with(SceneTextures::setup);
        graph.add_node_with(ShadowResource::setup);
        graph.add_node_with(DirectionalShadowPass::setup);
        graph.add_node_with(PointShadowPass::setup);
        graph.add_node_with(SkyboxRender::setup);
        graph.add_node_with(MainPass::setup);
        graph.add_node_with(CompositePass::setup);

        graph.add_edge::<EnvironmentPrePass, SkyboxRender>();
        graph.add_edge::<SceneTextures, SkyboxRender>();
        graph.add_edge::<ShadowResource, DirectionalShadowPass>();
        graph.add_edge::<ShadowResource, PointShadowPass>();
        graph.add_edge::<DirectionalShadowPass, MainPass>();
        graph.add_edge::<PointShadowPass, MainPass>();
        graph.add_edge::<SkyboxRender, MainPass>();
        graph.add_edge::<MainPass, CompositePass>();
    }
}
