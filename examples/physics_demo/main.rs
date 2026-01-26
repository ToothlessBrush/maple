use maple::prelude::*;

fn main() {
    App::new(Config {
        window_mode: WindowMode::FullScreen,
        ..Default::default()
    })
    .add_plugin(Core3D)
    .add_plugin(Physics3D)
    .load_scene(PhysicsScene)
    .run();
}

pub struct PhysicsScene;

impl SceneBuilder for PhysicsScene {
    fn build(&mut self, assets: &AssetLibrary) -> Scene {
        let scene = Scene::default();

        // Camera
        let camera = scene.spawn(
            "Camera",
            Camera3D::builder()
                .position(Vec3::new(-40.0, 40.0, -40.0))
                .orientation_vector(Vec3::ZERO - Vec3::new(-40.0, 40.0, -40.0))
                .far_plane(500.0)
                .build(),
        );
        camera
            .on::<Ready>(|ctx| {
                ctx.get_resource_mut::<Input>().set_cursor_locked(true);
            })
            .on::<Update>(Camera3D::free_fly(5.0, 1.0));

        // Light
        scene.spawn(
            "Sun",
            DirectionalLight::builder()
                .direction(Vec3::new(-1.0, -1.0, -1.0))
                .intensity(2.0)
                .build(),
        );

        // Ground - static rigid body with box collider
        let ground = scene.spawn(
            "Ground",
            RigidBody3DBuilder::fixed()
                .position(Vec3::new(0.0, -1.0, 0.0))
                .build(),
        );
        ground.spawn_child(
            "mesh",
            Mesh3D::cube()
                .scale(Vec3 {
                    x: 10000.0,
                    y: 1.0,
                    z: 10000.0,
                })
                .material(
                    MaterialProperties::default()
                        .with_base_color_factor(Color::GREY)
                        .with_roughness_factor(0.3)
                        .with_metallic_factor(0.7),
                )
                .build(),
        );
        ground.spawn_child(
            "collider",
            Collider3DBuilder::cuboid(5000.0, 1.0, 5000.0).build(),
        );

        scene.spawn(
            "sky",
            Environment::new(assets.load("res/kloofendal_48d_partly_cloudy_puresky_4k.hdr"))
                .with_ibl_strength(0.5),
        );

        // createa a cube of half sized cubes
        for x in -5..5 {
            for y in 0..10 {
                for z in -5..5 {
                    let ball = scene.spawn(
                        format!("ballx{}y{}z{}", x, y, z),
                        RigidBody3DBuilder::dynamic()
                            .position(Vec3::new(x as f32, y as f32, z as f32))
                            .build(),
                    );
                    ball.spawn_child(
                        "mesh",
                        Mesh3D::cube()
                            .material(
                                MaterialProperties::default().with_base_color_factor(Color::RED),
                            )
                            .scale_factor(0.5)
                            .build(),
                    );
                    ball.spawn_child("collider", Collider3DBuilder::cube(0.5).build());
                }
            }
        }

        let ball = scene.spawn(
            "ball",
            RigidBody3DBuilder::kinematic_velocity_based()
                .position(Vec3 {
                    x: -400.0,
                    y: 5.0,
                    z: -400.0,
                })
                .linear_velocity(Vec3 {
                    x: 100.0,
                    y: 0.0,
                    z: 100.0,
                })
                .build(),
        );
        ball.spawn_child(
            "mesh",
            Mesh3D::sphere()
                .material(MaterialProperties::default().with_base_color_factor(Color::RED))
                .build(),
        );
        ball.spawn_child("collider", Collider3DBuilder::ball(1.0).build());

        scene
    }
}
