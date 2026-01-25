use std::f32::consts::PI;

use maple::prelude::*;
use maple_3d::prelude::Environment;
use maple_engine::asset::AssetLibrary;
use maple_physics::resource::ColliderEnter;

fn main() {
    App::new(Config::default())
        .add_plugin(Core3D)
        .add_plugin(Physics3D)
        .load_scene(MainScene)
        .run();
}

pub struct MainScene;

impl SceneBuilder for MainScene {
    fn build(&mut self, assets: &AssetLibrary) -> Scene {
        let scene = Scene::default();

        // scene.spawn(
        //     "skybox",
        //     Environment::new(Path::new("res/kloofendal_48d_partly_cloudy_puresky_4k.hdr"))
        //         .with_ibl_strength(1.0),
        // );

        scene
            .spawn(
                "Camera",
                Camera3D::builder()
                    .position((-10.0, 1.0, 0.0))
                    .far_plane(100.0)
                    .orientation_vector(
                        Vec3::ZERO
                            - Vec3 {
                                x: -10.0,
                                y: 1.0,
                                z: 0.0,
                            },
                    )
                    .fov(PI / 2.0)
                    .build(),
            )
            .on::<Ready>(|ctx| {
                ctx.game.get_resource_mut::<Input>().set_cursor_locked(true);
            })
            .on::<Update>(Camera3D::free_fly(1.0, 1.0));

        scene.spawn(
            "mesh",
            Mesh3D::smooth_sphere()
                .material(
                    MaterialProperties::default()
                        .with_base_color_factor(Color::RED)
                        .with_emissive_factor(Vec3 {
                            x: 10.0,
                            y: 0.0,
                            z: 0.0,
                        }),
                )
                .build(),
        );

        scene.spawn(
            "ground",
            Mesh3D::plane()
                .position((0.0, -1.0, 0.0))
                .scale_factor(9.0)
                .material(MaterialProperties::default().with_base_color_factor(Color::CYAN))
                .build(),
        );

        scene
    }
}
