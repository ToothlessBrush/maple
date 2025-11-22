use maple::prelude::*;
use maple_3d::{
    components::material::MaterialProperties,
    nodes::{camera::Camera3D, directional_light::DirectionalLight, mesh::Mesh3D},
};
use maple_engine::{
    components::event_reciever::{Ready, Update},
    utils::color,
};
use maple_physics::nodes::{Collider3DBuilder, RigidBody3DBuilder};

pub struct PhysicsScene;

impl SceneBuilder for PhysicsScene {
    fn build(&mut self) -> Scene {
        let mut scene = Scene::default();

        // Camera
        scene.add(
            "Camera",
            Camera3D::builder()
                .position(Vec3::new(-10.0, 5.0, 10.0))
                .orientation_vector(Vec3::ZERO - Vec3::new(-10.0, 5.0, 10.0))
                .far_plane(1000.0)
                .on(Ready, |ctx: &mut GameContext| {
                    ctx.get_resource_mut::<InputManager>()
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
                .direction(Vec3::new(-1.0, -1.0, -0.5))
                .intensity(1.0)
                .build(),
        );

        // Ground - static rigid body with box collider
        scene.add(
            "Ground",
            RigidBody3DBuilder::fixed()
                .position(Vec3::new(0.0, -5.0, 0.0))
                .add_child(
                    "mesh",
                    Mesh3D::cube()
                        .scale(Vec3 {
                            x: 200.0,
                            y: 1.0,
                            z: 200.0,
                        })
                        .material(MaterialProperties::default().with_base_color_factor(color::CYAN))
                        .build(),
                )
                .add_child(
                    "collider",
                    Collider3DBuilder::cuboid(100.0, 0.5, 100.0).build(),
                )
                .build(),
        );

        for i in 1..=100 {
            scene.add(
                &format!("ball{i}"),
                RigidBody3DBuilder::dynamic()
                    .position(Vec3::new(0.0, i as f32 * 10.0, 0.0))
                    .add_child(
                        "mesh",
                        Mesh3D::cube()
                            .material(
                                MaterialProperties::default()
                                    .with_base_color_factor(color::RED)
                                    .with_metallic_factor(i as f32 / 100.0),
                            )
                            .scale_factor(i as f32 / 10.0)
                            .build(),
                    )
                    .add_child(
                        "collider",
                        Collider3DBuilder::cube((i as f32 / 10.0) * 0.5).build(),
                    )
                    .ccd_enabled(true)
                    .build(),
            );
        }

        scene
    }
}
