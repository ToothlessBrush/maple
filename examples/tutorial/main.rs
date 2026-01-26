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

        let ball = scene.spawn(
            "ball",
            RigidBody3DBuilder::dynamic()
                .position((0.0, 30.0, 0.0))
                .build(),
        );

        let collider = ball.spawn_child(
            "collider",
            Collider3DBuilder::ball(1.0).restitution(1.0).build(),
        );

        // Register collision event AFTER.spawning to scene
        collider.on::<ColliderEnter>(|ctx| {
            println!(
                "boing! {:?}",
                ctx.game
                    .scene
                    .get::<Collider3D>(ctx.event.other)
                    .unwrap()
                    .name()
            )
        });

        ball.spawn_child(
            "mesh",
            Mesh3D::smooth_sphere()
                .material(MaterialProperties::default().with_base_color_factor(Color::RED))
                .build(),
        );

        let floor = scene.spawn(
            "floor",
            RigidBody3DBuilder::fixed()
                .position((0.0, -2.0, 0.0))
                .build(),
        );

        floor.spawn_child(
            "Collider",
            Collider3DBuilder::cuboid(10.0, 1.0, 10.0).build(),
        );

        scene.spawn(
            "ground",
            Mesh3D::plane()
                .position((0.0, -1.0, 0.0))
                .scale_factor(9.0)
                .material(MaterialProperties::default().with_base_color_factor(Color::CYAN))
                .build(),
        );

        scene.spawn(
            "direct",
            DirectionalLight::builder()
                .direction((-1.0, -1.0, -1.0))
                .intensity(10.0)
                .bias(0.0001)
                .build(),
        );

        scene
    }
}
