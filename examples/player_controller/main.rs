use maple::prelude::*;

fn main() {
    App::new(Config {
        window_mode: WindowMode::FullScreen,
        ..Default::default()
    })
    .add_plugin(Core3D)
    .add_plugin(Physics3D)
    .load_scene(PlayerScene)
    .run();
}

#[derive(Clone)]
struct PlayerController {
    move_speed: f32,
    jump_force: f32,
    grounded: bool,
}

impl Default for PlayerController {
    fn default() -> Self {
        Self {
            move_speed: 10.0,
            jump_force: 10.0,
            grounded: false,
        }
    }
}

pub struct PlayerScene;

impl SceneBuilder for PlayerScene {
    fn build(&mut self) -> Scene {
        let scene = Scene::default();

        // Sun light
        scene.spawn(
            "Sun",
            DirectionalLight::builder()
                .direction(Vec3::new(-1.0, -1.0, -0.5))
                .intensity(1.0)
                .build(),
        );

        // Ground - large static platform
        let ground = scene.spawn(
            "Ground",
            RigidBody3DBuilder::fixed()
                .position(Vec3::new(0.0, -1.0, 0.0))
                .build(),
        );
        ground.spawn_child(
            "mesh",
            Mesh3D::cube()
                .scale(Vec3::new(10000.0, 1.0, 10000.0))
                .material(MaterialProperties::default().with_base_color_factor(Color::GREY))
                .build(),
        );
        ground.spawn_child(
            "collider",
            Collider3DBuilder::cuboid(5000.0, 0.5, 5000.0).build(),
        );

        // Add some platforms to jump on
        let plat1 = scene.spawn(
            "Platform1",
            RigidBody3DBuilder::fixed()
                .position(Vec3::new(3.0, 0.5, 3.0))
                .build(),
        );
        plat1.spawn_child(
            "mesh",
            Mesh3D::cube()
                .scale(Vec3::new(1.5, 0.25, 1.5))
                .material(MaterialProperties::default().with_base_color_factor(Color::YELLOW))
                .build(),
        );
        plat1.spawn_child(
            "collider",
            Collider3DBuilder::cuboid(1.5, 0.25, 1.5).build(),
        );

        let plat2 = scene.spawn(
            "Platform2",
            RigidBody3DBuilder::fixed()
                .position(Vec3::new(-5.0, 2.0, 8.0))
                .build(),
        );
        plat2.spawn_child(
            "mesh",
            Mesh3D::cube()
                .scale(Vec3::new(1.5, 0.25, 1.5))
                .material(MaterialProperties::default().with_base_color_factor(Color::YELLOW))
                .build(),
        );
        plat2.spawn_child(
            "collider",
            Collider3DBuilder::cuboid(1.5, 0.25, 1.5).build(),
        );

        let plat3 = scene.spawn(
            "Platform3",
            RigidBody3DBuilder::fixed()
                .position(Vec3::new(8.0, 3.5, -3.0))
                .build(),
        );
        plat3.spawn_child(
            "mesh",
            Mesh3D::cube()
                .scale(Vec3::new(3.0, 0.5, 3.0))
                .material(MaterialProperties::default().with_base_color_factor(Color::YELLOW))
                .build(),
        );
        plat3.spawn_child("collider", Collider3DBuilder::cuboid(3.0, 0.5, 3.0).build());

        // Add some obstacles
        for i in 0..5 {
            let x = (i as f32 - 2.0) * 4.0;
            let obsticle = scene.spawn(
                format!("Obstacle{}", i),
                RigidBody3DBuilder::fixed()
                    .position(Vec3::new(x, 0.5, -5.0))
                    .build(),
            );
            obsticle.spawn_child(
                "mesh",
                Mesh3D::cube()
                    .scale(Vec3::new(1.0, 2.0, 1.0))
                    .material(MaterialProperties::default().with_base_color_factor(Color::RED))
                    .build(),
            );
            obsticle.spawn_child("collider", Collider3DBuilder::cuboid(1.0, 2.0, 1.0).build());
        }

        // Player - dynamic rigid body with capsule collider
        let player = scene.spawn(
            "Player",
            RigidBody3DBuilder::dynamic()
                .position(Vec3::new(0.0, 5.0, 0.0))
                // Lock rotation so the player doesn't tumble
                .lock_rotations()
                .linear_damping(2.0) // Add some damping for better control
                .ccd_enabled(true) // Enable continuous collision detection
                .build(),
        );
        player.on::<Update>(move |ctx| {
            let input = ctx.game.get_resource::<Input>();

            let mut controller = PlayerController::default();

            let mut node = ctx.node.write();

            // Get camera for direction (camera is a child of the player)
            let camera_transform =
                if let Some(camera) = ctx.game.scene.get_by_name::<Camera3D>("Camera") {
                    *camera.write().get_transform()
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
            controller.grounded = node.velocity.y.abs() < 0.1 && node.transform.position.y < 10.0;

            // Jump with space bar (only when grounded)
            if input.key_just_pressed.contains(&KeyCode::Space) && controller.grounded {
                node.velocity.y = controller.jump_force;
            }
        });

        player.spawn_child(
            "Body",
            Mesh3D::cube()
                .scale(Vec3::new(0.5, 1.0, 0.5))
                .material(MaterialProperties::default().with_base_color_factor(Color::BLUE))
                .build(),
        );
        player.spawn_child(
            "collider",
            // Capsule collider for smooth movement over obstacles
            Collider3DBuilder::capsule_y(0.5, 0.5).friction(0.3).build(),
        );
        let camera = player.spawn_child(
            "Camera",
            Camera3D::builder()
                .position(Vec3::new(0.0, 0.5, 0.0))
                .far_plane(100.0)
                .fov(1.57)
                .build(),
        );
        camera.on::<Ready>(|ctx| {
            // Lock cursor for FPS-style controls
            ctx.game.get_resource_mut::<Input>().set_cursor_locked(true);
        });
        camera.on::<Update>(Camera3D::free_look(1.0));

        scene
    }
}
