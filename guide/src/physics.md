# Physics

Maple includes a 3D physics system for realistic object interactions, collisions, and movement.

## Enabling Physics

Add the `Physics3D` plugin to your app:

```rust
App::new(Config::default())
    .add_plugin(Core3D)
    .add_plugin(Physics3D)  // Enable physics
    .load_scene(MyScene)
    .run();
```

## Rigid Bodies

Rigid bodies are physical objects that can move and collide. There are two types:

### Dynamic Bodies

Dynamic bodies are affected by gravity and forces:

```rust
scene.add(
    "Ball",
    RigidBody3DBuilder::dynamic()
        .position((0.0, 10.0, 0.0))
        .add_child(
            "collider",
            Collider3DBuilder::ball(1.0).build(),
        )
        .add_child(
            "mesh",
            Mesh3D::smooth_sphere()
                .material(MaterialProperties::default().with_base_color_factor(Color::RED))
                .build(),
        )
        .build(),
);
```

### Fixed Bodies

Fixed bodies never move and are used for static environment objects:

```rust
scene.add(
    "Ground",
    RigidBody3DBuilder::fixed()
        .position((0.0, 0.0, 0.0))
        .add_child(
            "collider",
            Collider3DBuilder::cuboid(10.0, 1.0, 10.0).build(),
        )
        .build(),
);
```

## Colliders

Colliders define the physical shape for collision detection. They must be added as children of rigid bodies.

### Collider Shapes

```rust
// Box collider (half-extents: width/2, height/2, depth/2)
Collider3DBuilder::cuboid(5.0, 1.0, 5.0).build()

// Sphere collider
Collider3DBuilder::ball(1.0).build()  // radius

// Capsule collider (good for characters)
Collider3DBuilder::capsule_y(height, radius).build()
```

### Collider Properties

```rust
Collider3DBuilder::cuboid(1.0, 1.0, 1.0)
    .restitution(0.8)   // Bounciness (0.0 = no bounce, 1.0 = perfect bounce)
    .friction(0.5)      // Surface friction (0.0 = ice, 1.0 = rubber)
    .build()
```

## Rigid Body Properties

### Position and Movement

```rust
RigidBody3DBuilder::dynamic()
    .position((x, y, z))           // Initial position
    .position(Vec3::new(x, y, z))  // Or use Vec3
    .velocity(Vec3::new(0.0, 0.0, 0.0))  // Initial velocity
    .build()
```

### Constraints

```rust
RigidBody3DBuilder::dynamic()
    .lock_rotations()      // Prevent rotation (useful for characters)
    .linear_damping(2.0)   // Slow down over time (air resistance)
    .angular_damping(1.0)  // Rotational damping
    .build()
```

### Advanced Options

```rust
RigidBody3DBuilder::dynamic()
    .ccd_enabled(true)     // Continuous collision detection (prevents fast objects from passing through walls)
    .build()
```

## Modifying Physics in Behaviors

You can modify a rigid body's velocity and properties in update behaviors:

```rust
RigidBody3DBuilder::dynamic()
    .on(Update, |node: &mut RigidBody3D, ctx: &GameContext| {
        let input = ctx.get_resource::<InputManager>().unwrap();

        // Modify velocity directly
        if input.keys.contains(&KeyCode::KeyW) {
            node.velocity.z = 5.0;
        }

        // Jump
        if input.key_just_pressed.contains(&KeyCode::Space) {
            node.velocity.y = 10.0;
        }

        // Access position
        let pos = node.transform.position;
    })
    .build()
```

## Complete Physics Example

```rust
impl SceneBuilder for PhysicsScene {
    fn build(&mut self) -> Scene {
        let mut scene = Scene::default();

        // Camera
        scene.add(
            "Camera",
            Camera3D::builder()
                .position((-10.0, 5.0, 0.0))
                .orientation_vector(Vec3::new(1.0, -0.5, 0.0))
                .build(),
        );

        // Light
        scene.add(
            "Sun",
            DirectionalLight::builder()
                .direction((-1.0, -1.0, -1.0))
                .intensity(1.0)
                .build(),
        );

        // Fixed ground
        scene.add(
            "Ground",
            RigidBody3DBuilder::fixed()
                .position((0.0, -1.0, 0.0))
                .add_child(
                    "collider",
                    Collider3DBuilder::cuboid(10.0, 0.5, 10.0).build(),
                )
                .add_child(
                    "mesh",
                    Mesh3D::cube()
                        .scale(Vec3::new(10.0, 0.5, 10.0))
                        .material(MaterialProperties::default().with_base_color_factor(Color::GREY))
                        .build(),
                )
                .build(),
        );

        // Dynamic bouncing ball
        scene.add(
            "Ball",
            RigidBody3DBuilder::dynamic()
                .position((0.0, 10.0, 0.0))
                .add_child(
                    "collider",
                    Collider3DBuilder::ball(1.0)
                        .restitution(0.9)  // Very bouncy
                        .build(),
                )
                .add_child(
                    "mesh",
                    Mesh3D::smooth_sphere()
                        .material(MaterialProperties::default().with_base_color_factor(Color::RED))
                        .build(),
                )
                .build(),
        );

        scene
    }
}
```

## Physics Best Practices

1. **Always add both mesh and collider** - The mesh is what you see, the collider is what physics uses
2. **Match shapes** - Make collider shapes roughly match your visual mesh
3. **Use CCD for fast objects** - Enable `.ccd_enabled(true)` for objects that move quickly
4. **Lock rotations for characters** - Use `.lock_rotations()` to prevent player characters from tumbling
5. **Add damping** - Use `.linear_damping()` for more controllable movement

## Next Steps

- See a complete example in the [Player Controller Guide](examples/player-controller.md)
- Learn more about [Behaviors](behavior.md) to create interactive physics objects
