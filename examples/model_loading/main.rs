use std::f32::consts::PI;

use maple::prelude::*;
use maple_3d::gltf::GltfScene;

fn main() {
    App::new(Config::default())
        .add_plugin(Core3D)
        .load_scene(MainScene)
        .run();
}

pub struct MainScene;

impl SceneBuilder for MainScene {
    fn build(&mut self, assets: &AssetLibrary) -> Scene {
        let scene = Scene::default();

        scene.spawn(
            "skybox",
            Environment::new(assets.load("res/kloofendal_48d_partly_cloudy_puresky_4k.hdr"))
                .with_ibl_strength(0.2),
        );

        scene
            .spawn(
                "Camera",
                Camera3D::builder()
                    .fov(PI / 2.0)
                    .exposure(0.5)
                    .position(Vec3 {
                        x: -10.0,
                        y: 1.0,
                        z: 0.0,
                    })
                    .far_plane(100.0)
                    .near_plane(0.01)
                    .look_at(Vec3::ZERO)
                    .build(),
            )
            .on::<Update>(Camera3D::free_fly(1.0, 0.5))
            // .on::<Update>(|ctx| {
            //     ctx.node_mut().look_at(Vec3::ZERO);
            // })
            .on::<FixedUpdate>(|ctx| {
                let input = ctx.get_resource::<Input>();

                if input.keys.contains(&KeyCode::ArrowUp) {
                    ctx.node.write().exposure += 0.01
                }
                if input.keys.contains(&KeyCode::ArrowDown) {
                    ctx.node.write().exposure -= 0.01
                }
            });

        scene.spawn(
            "direct",
            DirectionalLight::builder()
                .direction((-0.5, -1.0, -0.5))
                .intensity(10.0)
                .build(),
        );

        let sponza =
            assets.load::<GltfScene>("res/models/main_sponza/NewSponza_Main_glTF_003.gltf");
        let curtains =
            assets.load::<GltfScene>("res/models/pkg_a_curtains/NewSponza_Curtains_glTF.gltf");
        let ivy = assets.load::<GltfScene>("res/models/pkg_b_ivy/NewSponza_IvyGrowth_glTF.gltf");

        let model = scene.spawn("model", Empty::builder().scale_factor(1.0).build());
        model.merge_asset(sponza);
        model.merge_asset(curtains);
        model.merge_asset(ivy);

        scene
    }
}
