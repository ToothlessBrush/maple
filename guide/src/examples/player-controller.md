# Player Controller Example

This guide walks through building a complete first-person player controller with physics-based movement.

## What We'll Build

A first-person character controller with:
- WASD movement
- Mouse look
- Jump mechanics
- Physics-based collision
- A simple test environment

## Full Code

```rust
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
                        .material(MaterialProperties::default()
                            .with_base_color_factor(Color::GREY))
                        .build(),
                )
                .add_child(
                    "collider",
                    Collider3DBuilder::cuboid(5000.0, 0.5, 5000.0).build(),
                )
                .build(),
        );

        // Player
        scene.add(
            "Player",
            RigidBody3DBuilder::dynamic()
                .position(Vec3::new(0.0, 5.0, 0.0))
                .lock_rotations()          // Prevent tumbling
                .linear_damping(2.0)       // Add air resistance
                .ccd_enabled(true)         // Prevent tunneling
                .add_child(
                    "Body",
                    Mesh3D::cube()
                        .scale(Vec3::new(0.5, 1.0, 0.5))
                        .material(MaterialProperties::default()
                            .with_base_color_factor(Color::BLUE))
                        .build(),
                )
                .add_child(
                    "collider",
                    Collider3DBuilder::capsule_y(0.5, 0.5)
                        .friction(0.3)
                        .build(),
                )
                .add_child(
                    "Camera",
                    Camera3D::builder()
                        .position(Vec3::new(0.0, 0.5, 0.0))
                        .far_plane(100.0)
                        .fov(1.57)
                        .on(Ready, |ctx: &mut GameContext| {
                            // Lock cursor for FPS controls
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
                .on(Update, player_movement)
                .build(),
        );

        scene
    }
}

fn player_movement(node: &mut RigidBody3D, ctx: &mut GameContext) {
    let input = ctx.get_resource::<InputManager>().unwrap();

    // Configuration
    let move_speed = 10.0;
    let jump_force = 10.0;

    // Get camera direction from child
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

    // Apply horizontal movement
    let target_velocity = movement * move_speed;
    node.velocity.x = target_velocity.x;
    node.velocity.z = target_velocity.z;

    // Simple ground detection
    let grounded = node.velocity.y.abs() < 0.1 && node.transform.position.y < 10.0;

    // Jump
    if input.key_just_pressed.contains(&KeyCode::Space) && grounded {
        node.velocity.y = jump_force;
    }
}
```

## Breaking It Down

### 1. App Setup

```rust
App::new(Config {
    window_mode: WindowMode::FullScreen,
    ..Default::default()
})
.add_plugin(Core3D)
.add_plugin(Physics3D)
.load_scene(PlayerScene)
.run();
```

- Creates a fullscreen application
- Enables 3D rendering and physics
- Loads our player scene

### 2. Scene Environment

The scene includes basic lighting and a ground plane:

```rust
// Directional light for visibility
scene.add("Sun", DirectionalLight::builder()...);

// Fixed ground platform
scene.add("Ground", RigidBody3DBuilder::fixed()...);
```

### 3. Player Structure

The player is a rigid body with three children:

```rust
RigidBody3DBuilder::dynamic()
    .add_child("Body", Mesh3D::cube()...)      // Visual representation
    .add_child("collider", Collider3DBuilder...)  // Physics collision
    .add_child("Camera", Camera3D::builder()...)  // First-person view
```

### 4. Physics Configuration

```rust
.lock_rotations()      // Prevents character from falling over
.linear_damping(2.0)   // Adds resistance to movement
.ccd_enabled(true)     // Prevents tunneling through objects
```

- `lock_rotations()` is crucial - without it, the player will tumble when moving
- `linear_damping()` makes movement feel more controlled
- `ccd_enabled()` prevents the player from passing through walls at high speeds

### 5. Camera Setup

```rust
Camera3D::builder()
    .on(Ready, |ctx: &mut GameContext| {
        ctx.get_resource_mut::<InputManager>()
            .unwrap()
            .set_cursor_locked(true);  // Hide and lock cursor
    })
    .on(Update, |node: &mut Camera3D, ctx: &GameContext| {
        let input = ctx.get_resource::<InputManager>().unwrap();
        node.free_look(input, 0.01);  // Mouse look
    })
```

- `Ready` event locks the cursor when the game starts
- `Update` event applies mouse look every frame

### 6. Movement System

The movement function:

1. **Gets camera orientation** from the child node
2. **Projects movement onto XZ plane** to keep movement horizontal
3. **Handles WASD input** relative to camera direction
4. **Normalizes diagonal movement** so you don't move faster diagonally
5. **Applies velocity directly** to the rigid body
6. **Handles jumping** with basic ground detection

```rust
// Get camera direction
let forward = camera_transform.get_forward_vector();
let right = camera_transform.get_right_vector();

// Keep movement horizontal
let forward_xz = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
let right_xz = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();
```

## Customization

### Adjust Movement Speed

```rust
let move_speed = 10.0;  // Increase for faster movement
let jump_force = 10.0;  // Increase for higher jumps
```

### Change Camera Sensitivity

```rust
node.free_look(input, 0.01);  // Increase value for faster turning
```

### Improve Ground Detection

The example uses simple ground detection. For production, you'd want to:
- Use raycasts to detect ground
- Check collision contacts
- Implement a state machine for grounded/airborne states

## Adding Features

### Sprint

```rust
let move_speed = if input.keys.contains(&KeyCode::ShiftLeft) {
    20.0  // Sprint speed
} else {
    10.0  // Walk speed
};
```

### Platforms and Obstacles

Add more fixed rigid bodies to create a platforming environment:

```rust
scene.add(
    "Platform",
    RigidBody3DBuilder::fixed()
        .position(Vec3::new(5.0, 2.0, 5.0))
        .add_child("mesh", Mesh3D::cube()...)
        .add_child("collider", Collider3DBuilder::cuboid(2.0, 0.5, 2.0).build())
        .build(),
);
```

## Running the Example

The complete working example is available in `examples/player_controller/main.rs`. Run it with:

```bash
cargo run --example player_controller
```

## Next Steps

- Add more complex environments
- Implement better ground detection with raycasts
- Add animations for the player mesh
- Create a state machine for player states (idle, walking, jumping, falling)
- Add sound effects and particle effects
