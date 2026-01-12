use std::f32::consts::PI;

use maple::prelude::*;
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
    fn build(&mut self) -> Scene {
        let scene = Scene::default();

        // scene.add(
        //     "skybox",
        //     Environment::new(Path::new("res/kloofendal_48d_partly_cloudy_puresky_4k.hdr"))
        //         .with_ibl_strength(1.0),
        // );

        scene.add(
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
                .on::<Ready>(|ctx| {
                    ctx.game.get_resource_mut::<Input>().set_cursor_locked(true);
                })
                .fov(PI / 2.0)
                .on::<Update>(Camera3D::free_fly(1.0, 1.0))
                .build(),
        );

        let ball = scene.add(
            "ball",
            RigidBody3DBuilder::dynamic()
                .position((0.0, 30.0, 0.0))
                .build(),
        );

        ball.add_child(
            "collider",
            Collider3DBuilder::ball(1.0)
                .restitution(1.0)
                .on::<ColliderEnter>(|ctx| {
                    println!(
                        "boing! {:?}",
                        ctx.game
                            .scene
                            .get::<Collider3D>(ctx.event.other)
                            .unwrap()
                            .name()
                    )
                })
                .build(),
        );

        ball.add_child(
            "mesh",
            Mesh3D::smooth_sphere()
                .material(
                    MaterialProperties::default()
                        .with_base_color_factor(Color::RED)
                        .with_metallic_factor(0.5)
                        .with_roughness_factor(0.5),
                )
                .build(),
        );

        let floor = scene.add(
            "floor",
            RigidBody3DBuilder::fixed()
                .position((0.0, -2.0, 0.0))
                .build(),
        );

        floor.add_child(
            "Collider",
            Collider3DBuilder::cuboid(10.0, 1.0, 10.0).build(),
        );

        scene.add(
            "ground",
            Mesh3D::plane()
                .position((0.0, -1.0, 0.0))
                .scale_factor(9.0)
                .build(),
        );

        scene.add(
            "direct",
            DirectionalLight::builder()
                .direction((-1.0, -1.0, -1.0))
                .intensity(1.0)
                .bias(0.0001)
                .build(),
        );

        scene
    }
}
