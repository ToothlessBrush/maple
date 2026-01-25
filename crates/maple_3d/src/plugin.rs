use maple_app::Plugin;

use crate::{
    gltf::GltfSceneLoader,
    render_passes::{
        bloom::BloomPass, composite_pass::CompositePass,
        directional_shadow_pass::DirectionalShadowPass, environment::EnvironmentPrePass,
        main_pass::MainPass, point_shadow_pass::PointShadowPass, scene_textures::SceneTextures,
        shadow_resource::ShadowResource, skybox::SkyboxRender,
    },
};

pub struct Core3D;

impl Plugin for Core3D {
    fn setup(&self, app: &mut maple_app::App<maple_app::Init>) {
        app.context_mut().assets.register_loader(GltfSceneLoader);
    }

    fn ready(&self, app: &mut maple_app::App<maple_app::Running>) {
        let mut graph = app.renderer_mut().graph();

        graph.add_node_with(EnvironmentPrePass::setup);
        graph.add_node_with(SceneTextures::setup);
        graph.add_node_with(ShadowResource::setup);
        graph.add_node_with(DirectionalShadowPass::setup);
        graph.add_node_with(PointShadowPass::setup);
        graph.add_node_with(SkyboxRender::setup);
        graph.add_node_with(MainPass::setup);
        graph.add_node_with(CompositePass::setup);
        graph.add_node_with(BloomPass::setup);

        graph.add_edge::<EnvironmentPrePass, SkyboxRender>();
        graph.add_edge::<SceneTextures, SkyboxRender>();
        graph.add_edge::<ShadowResource, DirectionalShadowPass>();
        graph.add_edge::<ShadowResource, PointShadowPass>();
        graph.add_edge::<DirectionalShadowPass, MainPass>();
        graph.add_edge::<PointShadowPass, MainPass>();
        graph.add_edge::<SkyboxRender, MainPass>();
        graph.add_edge::<MainPass, BloomPass>();
        graph.add_edge::<BloomPass, CompositePass>();
        graph.add_edge::<MainPass, CompositePass>();
    }
}
