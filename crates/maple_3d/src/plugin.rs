use maple_app::Plugin;

use crate::{
    assets::{
        material::{MaterialLoader, MaterialPipelineCache},
        mesh::Mesh3DLoader,
    },
    gltf::GltfSceneLoader,
    render_passes::{
        bloom::BloomPass, collect_mesh::CollectMesh, composite_pass::CompositePass,
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

        graph.setup_and_add_node::<EnvironmentPrePass>();
        graph.setup_and_add_node::<SceneTextures>();
        graph.setup_and_add_node::<CollectMesh>();
        graph.setup_and_add_node::<ShadowResource>();
        graph.setup_and_add_node::<DirectionalShadowPass>();
        graph.setup_and_add_node::<PointShadowPass>();
        graph.setup_and_add_node::<SkyboxRender>();
        graph.setup_and_add_node::<MainPass>();
        graph.setup_and_add_node::<CompositePass>();
        graph.setup_and_add_node::<BloomPass>();

        graph.add_edge::<CollectMesh, DirectionalShadowPass>();
        graph.add_edge::<CollectMesh, PointShadowPass>();
        graph.add_edge::<CollectMesh, MainPass>();
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
