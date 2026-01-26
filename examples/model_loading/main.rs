use std::f32::consts::PI;

use maple::prelude::*;

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
            .on::<Update>(Camera3D::free_fly(1.0, 0.5))
            .on::<Update>(|ctx| {
                let input = ctx.get_resource::<Input>();

                if input.keys.contains(&KeyCode::ArrowUp) {
                    ctx.node.write().exposure += 0.01
                }
                if input.keys.contains(&KeyCode::ArrowDown) {
                    ctx.node.write().exposure -= 0.01
                }
            });

        let gltf = assets.load::<GltfScene>("res/DamagedHelmet.glb");

        let model = scene.spawn("model", Empty::builder().scale_factor(1.0).build());
        model.merge_asset(gltf);

        scene
    }
}
