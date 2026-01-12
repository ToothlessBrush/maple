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

#[derive(Clone)]
struct Manager {
    pub gravity: f32,
}

pub struct PhysicsScene;

impl SceneBuilder for PhysicsScene {
    fn build(&mut self) -> Scene {
        let mut scene = Scene::default();

        // Camera
        scene.add(
            "Camera",
            Camera3D::builder()
                .position(Vec3::new(-40.0, 40.0, -40.0))
                .orientation_vector(Vec3::ZERO - Vec3::new(-40.0, 40.0, -40.0))
                .far_plane(500.0)
                .on::<Ready>(|ctx| {
                    ctx.get_resource_mut::<Input>()
                        .unwrap()
                        .set_cursor_locked(true);
                })
                .on(Update, Camera3D::free_fly(5.0, 1.0))
                .build(),
        );

        // Light
        scene.add(
            "Sun",
            DirectionalLight::builder()
                .direction(Vec3::new(-1.0, -0.5, 1.0))
                .intensity(1.0)
                .build(),
        );

        // Ground - static rigid body with box collider
        scene.add(
            "Ground",
            RigidBody3DBuilder::fixed()
                .position(Vec3::new(0.0, -1.0, 0.0))
                .add_child(
                    "mesh",
                    Mesh3D::cube()
                        .scale(Vec3 {
                            x: 10000.0,
                            y: 1.0,
                            z: 10000.0,
                        })
                        .material(MaterialProperties::default().with_base_color_factor(Color::CYAN))
                        .build(),
                )
                .add_child(
                    "collider",
                    Collider3DBuilder::cuboid(5000.0, 0.5, 5000.0).build(),
                )
                .build(),
        );

        for x in -5..5 {
            for y in 0..10 {
                for z in -5..5 {
                    scene.add(
                        &format!("ballx{}y{}z{}", x, y, z),
                        RigidBody3DBuilder::dynamic()
                            .position(Vec3::new(x as f32, y as f32, z as f32))
                            .add_child(
                                "mesh",
                                Mesh3D::cube()
                                    .material(
                                        MaterialProperties::default()
                                            .with_base_color_factor(Color::RED),
                                    )
                                    .build(),
                            )
                            .add_child("collider", Collider3DBuilder::cube(0.5).build())
                            .build(),
                    );
                }
            }
        }

        scene.add(
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
                .add_child(
                    "mesh",
                    Mesh3D::cube()
                        .material(MaterialProperties::default().with_base_color_factor(Color::RED))
                        .build(),
                )
                .add_child("collider", Collider3DBuilder::ball(0.5).build())
                .build(),
        );

        scene
    }
}
