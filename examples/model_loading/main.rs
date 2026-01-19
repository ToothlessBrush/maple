use std::{f32::consts::PI, path::Path, time::Duration};

use maple::prelude::*;
use maple_3d::{gltf::GltfScene, nodes::environment::Environment};
use maple_engine::asset::AssetLibrary;

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
            Environment::new(Path::new("res/kloofendal_48d_partly_cloudy_puresky_4k.hdr"))
                .with_ibl_strength(0.2),
        );

        scene
            .spawn(
                "Camera",
                Camera3D::builder()
                    .fov(PI / 2.0)
                    .position(Vec3 {
                        x: -10.0,
                        y: 1.0,
                        z: 0.0,
                    })
                    .far_plane(100.0)
                    .near_plane(0.01)
                    .orientation_vector(
                        Vec3::ZERO
                            - Vec3 {
                                x: -10.0,
                                y: 1.0,
                                z: 0.0,
                            },
                    )
                    .build(),
            )
            .on::<Ready>(|ctx| {
                ctx.game.get_resource_mut::<Input>().set_cursor_locked(true);
            })
            .on::<Update>(Camera3D::free_fly(1.0, 0.5));

        let gltf = assets.load::<GltfScene>("res/DamagedHelmet.glb");

        let model = scene.spawn("model", Empty::builder().scale_factor(1.0).build());
        model.merge_asset(gltf);

        scene.spawn(
            "direct",
            DirectionalLight::builder()
                .direction(Vec3 {
                    x: 0.0,
                    y: -1.0,
                    z: -1.0,
                })
                .color((1.0, 0.95, 0.8, 1.0))
                .intensity(2.0)
                .bias(0.0001)
                .build(),
        );

        scene
    }
}
