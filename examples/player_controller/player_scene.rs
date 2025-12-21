use maple::prelude::*;
use maple_3d::{
    components::material::MaterialProperties,
    nodes::{camera::Camera3D, directional_light::DirectionalLight, mesh::Mesh3D},
};
use maple_engine::{
    components::event_reciever::{Ready, Update},
    input::KeyCode,
    utils::color::Color,
};
use maple_physics::{
    nodes::{Collider3DBuilder, RigidBody3D, RigidBody3DBuilder},
    resource::Physics,
};

#[derive(Clone)]
struct PlayerController {
    move_speed: f32,
    jump_force: f32,
    camera_sensitivity: f32,
    grounded: bool,
}

impl Default for PlayerController {
    fn default() -> Self {
        Self {
            move_speed: 10.0,
            jump_force: 10.0,
            camera_sensitivity: 0.002,
            grounded: false,
        }
    }
}

pub struct PlayerScene;

impl SceneBuilder for PlayerScene {
    fn build(&mut self) -> Scene {
        let mut scene = Scene::default();

        // Sun light
        scene.add(
            "Sun",
            DirectionalLight::builder()
                .direction(Vec3::new(-1.0, -1.0, -0.5))
                .intensity(1.0)
                .build(),
        );

        // Ground - large static platform
        scene.add(
            "Ground",
            RigidBody3DBuilder::fixed()
                .position(Vec3::new(0.0, -1.0, 0.0))
                .add_child(
                    "mesh",
                    Mesh3D::cube()
                        .scale(Vec3::new(10000.0, 1.0, 10000.0))
                        .material(MaterialProperties::default().with_base_color_factor(Color::GREY))
                        .build(),
                )
                .add_child(
                    "collider",
                    Collider3DBuilder::cuboid(5000.0, 0.5, 5000.0).build(),
                )
                .build(),
        );

        // Add some platforms to jump on
        scene.add(
            "Platform1",
            RigidBody3DBuilder::fixed()
                .position(Vec3::new(5.0, 1.0, 5.0))
                .add_child(
                    "mesh",
                    Mesh3D::cube()
                        .scale(Vec3::new(3.0, 0.5, 3.0))
                        .material(
                            MaterialProperties::default().with_base_color_factor(Color::YELLOW),
                        )
                        .build(),
                )
                .add_child(
                    "collider",
                    Collider3DBuilder::cuboid(1.5, 0.25, 1.5).build(),
                )
                .build(),
        );

        scene.add(
            "Platform2",
            RigidBody3DBuilder::fixed()
                .position(Vec3::new(-5.0, 2.0, 8.0))
                .add_child(
                    "mesh",
                    Mesh3D::cube()
                        .scale(Vec3::new(3.0, 0.5, 3.0))
                        .material(
                            MaterialProperties::default().with_base_color_factor(Color::YELLOW),
                        )
                        .build(),
                )
                .add_child(
                    "collider",
                    Collider3DBuilder::cuboid(1.5, 0.25, 1.5).build(),
                )
                .build(),
        );

        scene.add(
            "Platform3",
            RigidBody3DBuilder::fixed()
                .position(Vec3::new(8.0, 3.5, -3.0))
                .add_child(
                    "mesh",
                    Mesh3D::cube()
                        .scale(Vec3::new(3.0, 0.5, 3.0))
                        .material(
                            MaterialProperties::default().with_base_color_factor(Color::YELLOW),
                        )
                        .build(),
                )
                .add_child(
                    "collider",
                    Collider3DBuilder::cuboid(1.5, 0.25, 1.5).build(),
                )
                .build(),
        );

        // Add some obstacles
        for i in 0..5 {
            let x = (i as f32 - 2.0) * 4.0;
            scene.add(
                &format!("Obstacle{}", i),
                RigidBody3DBuilder::fixed()
                    .position(Vec3::new(x, 0.5, -5.0))
                    .add_child(
                        "mesh",
                        Mesh3D::cube()
                            .scale(Vec3::new(1.0, 2.0, 1.0))
                            .material(
                                MaterialProperties::default().with_base_color_factor(Color::RED),
                            )
                            .build(),
                    )
                    .add_child("collider", Collider3DBuilder::cuboid(0.5, 1.0, 0.5).build())
                    .build(),
            );
        }

        // Player - dynamic rigid body with capsule collider
        scene.add(
            "Player",
            RigidBody3DBuilder::dynamic()
                .position(Vec3::new(0.0, 5.0, 0.0))
                // Lock rotation so the player doesn't tumble
                .lock_rotations()
                .linear_damping(2.0) // Add some damping for better control
                .ccd_enabled(true) // Enable continuous collision detection
                .add_child(
                    "Body",
                    Mesh3D::cube()
                        .scale(Vec3::new(0.5, 1.0, 0.5))
                        .material(MaterialProperties::default().with_base_color_factor(Color::BLUE))
                        .build(),
                )
                .add_child(
                    "collider",
                    // Capsule collider for smooth movement over obstacles
                    Collider3DBuilder::capsule_y(0.5, 0.5).friction(0.3).build(),
                )
                .add_child(
                    "Camera",
                    Camera3D::builder()
                        .position(Vec3::new(0.0, 0.5, 0.0))
                        .far_plane(1000.0)
                        .fov(1.57)
                        .on(Ready, |ctx: &mut GameContext| {
                            // Lock cursor for FPS-style controls
                            ctx.get_resource_mut::<InputManager>()
                                .unwrap()
                                .set_cursor_locked(true);
                        })
                        .on(Update, |node: &mut Camera3D, ctx: &GameContext| {
                            let input = ctx.get_resource::<InputManager>().unwrap();

                            node.free_look(input, 0.01);
                        })
                        .build(),
                )
                .on(
                    Update,
                    move |node: &mut RigidBody3D, ctx: &mut GameContext| {
                        let input = ctx.get_resource::<InputManager>().unwrap();

                        let mut controller = PlayerController::default();

                        // Get camera for direction (camera is a child of the player)
                        let camera_transform = if let Some(camera) =
                            node.get_children_mut().get_mut::<Camera3D>("Camera")
                        {
                            *camera.get_transform()
                        } else {
                            return;
                        };

                        // Calculate movement direction based on camera orientation
                        let forward = camera_transform.get_forward_vector();
                        let right = camera_transform.get_right_vector();

                        // Project directions onto XZ plane (keep movement horizontal)
                        let forward_xz = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
                        let right_xz = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();

                        let mut movement = Vec3::ZERO;

                        // WASD movement
                        if input.keys.contains(&KeyCode::KeyW) {
                            movement += forward_xz;
                        }
                        if input.keys.contains(&KeyCode::KeyS) {
                            movement -= forward_xz;
                        }
                        if input.keys.contains(&KeyCode::KeyA) {
                            movement += right_xz;
                        }
                        if input.keys.contains(&KeyCode::KeyD) {
                            movement -= right_xz;
                        }

                        // Normalize diagonal movement
                        if movement.length_squared() > 0.0 {
                            movement = movement.normalize();
                        }

                        // Apply horizontal movement by modifying velocity
                        let target_velocity = movement * controller.move_speed;
                        node.velocity.x = target_velocity.x;
                        node.velocity.z = target_velocity.z;

                        // Simple ground detection - check if Y velocity is near zero and we're not falling
                        controller.grounded =
                            node.velocity.y.abs() < 0.1 && node.transform.position.y < 10.0;

                        // Jump with space bar (only when grounded)
                        if input.key_just_pressed.contains(&KeyCode::Space) && controller.grounded {
                            node.velocity.y = controller.jump_force;
                        }
                    },
                )
                .build(),
        );

        scene
    }
}
