# Behavior System

Behaviors add logic and interactivity to your nodes using event handlers. This guide covers the `.on()` system.

## Event Handlers

Attach behaviors to nodes using the `.on()` method:

```rust
MyNode::builder()
    .on(EventType, |node, ctx| {
        // Your behavior code
    })
    .build()
```

## Event Types

### Ready

Called once when the node is first initialized:

```rust
Camera3D::builder()
    .on(Ready, |ctx: &mut GameContext| {
        // Initialize state, setup resources, etc.
        ctx.get_resource_mut::<InputManager>()
            .unwrap()
            .set_cursor_locked(true);
    })
    .build()
```

### Update

Called every frame during the game loop:

```rust
Camera3D::builder()
    .on(Update, |node: &mut Camera3D, ctx: &GameContext| {
        // Update logic runs every frame
        let input = ctx.get_resource::<InputManager>().unwrap();
        node.free_look(input, 0.01);
    })
    .build()
```

## Accessing Game Context

The `GameContext` provides access to engine resources:

```rust
.on(Update, |node: &mut MyNode, ctx: &GameContext| {
    // Get input
    let input = ctx.get_resource::<InputManager>().unwrap();

    // Get mutable input
    let input_mut = ctx.get_resource_mut::<InputManager>().unwrap();
})
```

## Input Handling

### Keyboard Input

Check key states using the `InputManager`:

```rust
.on(Update, |node: &mut MyNode, ctx: &GameContext| {
    let input = ctx.get_resource::<InputManager>().unwrap();

    // Check if key is currently pressed
    if input.keys.contains(&KeyCode::KeyW) {
        // W key is held down
    }

    // Check if key was just pressed this frame
    if input.key_just_pressed.contains(&KeyCode::Space) {
        // Space was just pressed
    }
})
```

Common key codes:
- `KeyCode::KeyW`, `KeyCode::KeyA`, `KeyCode::KeyS`, `KeyCode::KeyD`
- `KeyCode::Space`
- `KeyCode::Escape`
- `KeyCode::ShiftLeft`, `KeyCode::ControlLeft`

### Mouse Input

```rust
.on(Ready, |ctx: &mut GameContext| {
    // Lock cursor for FPS-style controls
    ctx.get_resource_mut::<InputManager>()
        .unwrap()
        .set_cursor_locked(true);
})
```

## Built-in Behaviors

Some nodes provide pre-built behaviors:

### Free Look Camera

```rust
Camera3D::builder()
    .on(Update, |node: &mut Camera3D, ctx: &GameContext| {
        let input = ctx.get_resource::<InputManager>().unwrap();
        node.free_look(input, 0.01);  // Mouse sensitivity
    })
    .build()
```

### Free Fly Camera

For a complete flying camera with WASD movement:

```rust
Camera3D::builder()
    .on(Ready, |ctx: &mut GameContext| {
        ctx.get_resource_mut::<InputManager>()
            .unwrap()
            .set_cursor_locked(true);
    })
    .on(Update, Camera3D::free_fly(1.0, 1.0))  // (move_speed, look_speed)
    .build()
```

## Accessing Node Children

Access child nodes within behaviors:

```rust
RigidBody3DBuilder::dynamic()
    .add_child(
        "Camera",
        Camera3D::builder()
            .position((0.0, 0.5, 0.0))
            .build(),
    )
    .on(Update, |node: &mut RigidBody3D, ctx: &GameContext| {
        // Get child camera's transform
        if let Some(camera) = node.get_children_mut().get_mut::<Camera3D>("Camera") {
            let camera_transform = *camera.get_transform();
            // Use camera transform for calculations
        }
    })
    .build()
```

## Complete Example

```rust
scene.add(
    "Player",
    RigidBody3DBuilder::dynamic()
        .position((0.0, 5.0, 0.0))
        .add_child(
            "Camera",
            Camera3D::builder()
                .position((0.0, 0.5, 0.0))
                .on(Ready, |ctx: &mut GameContext| {
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
        .on(Update, |node: &mut RigidBody3D, ctx: &GameContext| {
            let input = ctx.get_resource::<InputManager>().unwrap();

            // WASD movement
            let mut movement = Vec3::ZERO;
            if input.keys.contains(&KeyCode::KeyW) {
                movement.z += 1.0;
            }
            if input.keys.contains(&KeyCode::KeyS) {
                movement.z -= 1.0;
            }
            if input.keys.contains(&KeyCode::KeyA) {
                movement.x -= 1.0;
            }
            if input.keys.contains(&KeyCode::KeyD) {
                movement.x += 1.0;
            }

            // Apply movement
            node.velocity = movement.normalize_or_zero() * 5.0;

            // Jump
            if input.key_just_pressed.contains(&KeyCode::Space) {
                node.velocity.y = 10.0;
            }
        })
        .build(),
);
```

## Next Steps

- Implement [Physics](physics.md) for realistic movement
- Check out the [Player Controller Example](examples/player-controller.md) for a complete working example
