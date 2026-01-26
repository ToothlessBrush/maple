use std::f32::consts::PI;

use maple::prelude::*;

fn main() {
    App::new(Config::default())
        .add_plugin(Core3D)
        .add_plugin(Physics3D)
        .load_scene(MainScene)
        .run();
}

pub struct MainScene;

impl SceneBuilder for MainScene {
    fn build(&mut self, _assets: &AssetLibrary) -> Scene {
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
            Mesh3D::cube()
                .material(
                    MaterialProperties::default()
                        .with_base_color_factor(Color::RED)
                        .with_emissive_factor(Vec3 {
                            x: 10.0,
                            y: 0.0,
                            z: 0.0,
                        }),
                )
                .position((0.0, 0.0, -5.0))
                .build(),
        );

        scene.spawn(
            "mesh",
            Mesh3D::cube()
                .material(
                    MaterialProperties::default()
                        .with_base_color_factor(Color::GREEN)
                        .with_emissive_factor(Vec3 {
                            x: 0.0,
                            y: 10.0,
                            z: 0.0,
                        }),
                )
                .position((0.0, 0.0, 0.0))
                .build(),
        );

        scene.spawn(
            "mesh",
            Mesh3D::cube()
                .material(
                    MaterialProperties::default()
                        .with_base_color_factor(Color::BLUE)
                        .with_emissive_factor(Vec3 {
                            x: 0.0,
                            y: 0.0,
                            z: 10.0,
                        }),
                )
                .position((0.0, 0.0, 5.0))
                .build(),
        );

        scene.spawn(
            "mesh",
            Mesh3D::cube()
                .material(
                    MaterialProperties::default()
                        .with_base_color_factor(Color::WHITE)
                        .with_emissive_factor(Vec3 {
                            x: 10.0,
                            y: 10.0,
                            z: 10.0,
                        }),
                )
                .position((0.0, 0.0, 10.0))
                .build(),
        );

        scene.spawn(
            "mesh",
            Mesh3D::cube()
                .material(MaterialProperties::default().with_base_color_factor(Color::RED))
                .position((0.0, 5.0, -5.0))
                .build(),
        );

        scene.spawn(
            "mesh",
            Mesh3D::cube()
                .material(MaterialProperties::default().with_base_color_factor(Color::GREEN))
                .position((0.0, 5.0, 0.0))
                .build(),
        );

        scene.spawn(
            "mesh",
            Mesh3D::cube()
                .material(MaterialProperties::default().with_base_color_factor(Color::BLUE))
                .position((0.0, 5.0, 5.0))
                .build(),
        );

        scene.spawn(
            "mesh",
            Mesh3D::cube()
                .material(MaterialProperties::default().with_base_color_factor(Color::WHITE))
                .position((0.0, 5.0, 10.0))
                .build(),
        );

        scene.spawn("direct light", DirectionalLight::builder().build());

        scene
    }
}
