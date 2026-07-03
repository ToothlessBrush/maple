use maple_app::Plugin;

use crate::{
    assets::mesh::Mesh3DLoader,
    gltf::GltfSceneLoader,
    prelude::{MaterialLoader, MaterialPipelineCache},
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
        // assets
        let device = app.renderer().context.device().clone();
        let queue = app.renderer().context.queue().clone();
        let mipmap_generator = app.renderer().context.mipmap_generator().clone();
        app.context_mut()
            .assets
            .register_loader(Mesh3DLoader::new(device.clone()));
        app.context_mut()
            .assets
            .register_loader(MaterialLoader::new(device.clone()));
        app.context_mut()
            .assets
            .register_loader(GltfSceneLoader::new(device, queue, mipmap_generator));

        // resources
        app.context_mut()
            .insert_resource(MaterialPipelineCache::default());
    }

    fn ready(&self, app: &mut maple_app::App<maple_app::Running>) {
        let mut graph = app.renderer_mut().graph();

        graph.add_node_with_setup::<EnvironmentPrePass>();
        graph.add_node_with_setup::<SceneTextures>();
        graph.add_node_with_setup::<ShadowResource>();
        graph.add_node_with_setup::<DirectionalShadowPass>();
        graph.add_node_with_setup::<PointShadowPass>();
        graph.add_node_with_setup::<SkyboxRender>();
        graph.add_node_with_setup::<MainPass>();
        graph.add_node_with_setup::<CompositePass>();
        graph.add_node_with_setup::<BloomPass>();

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
