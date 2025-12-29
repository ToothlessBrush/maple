# Scene Creation

Scenes are collections of nodes that define your game world. This guide covers how to create and structure scenes.

## Defining a Scene

Scenes are created by implementing the `SceneBuilder` trait:

```rust
use maple::prelude::*;

pub struct MyScene;

impl SceneBuilder for MyScene {
    fn build(&mut self) -> Scene {
        let mut scene = Scene::default();

        // Add nodes here

        scene
    }
}
```

## Adding Nodes to a Scene

Nodes are added to the scene using the `add()` method. Each node needs a unique name:

```rust
scene.add(
    "my_node",  // Unique identifier
    MyNode::builder()
        .build()
);
```

## Scene Structure

A typical 3D scene includes:

1. **Camera** - Defines the viewpoint
2. **Lights** - Illuminates the scene
3. **Meshes/Models** - Visual objects
4. **Physics Bodies** - Objects with physics simulation (optional)

Example basic scene:

```rust
impl SceneBuilder for MainScene {
    fn build(&mut self) -> Scene {
        let mut scene = Scene::default();

        // Camera
        scene.add(
            "Camera",
            Camera3D::builder()
                .position((0.0, 5.0, -10.0))
                .orientation_vector(Vec3::new(0.0, -0.5, 1.0))
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

        // Ground mesh
        scene.add(
            "Ground",
            Mesh3D::plane()
                .position((0.0, 0.0, 0.0))
                .scale_factor(10.0)
                .build(),
        );

        scene
    }
}
```

## Loading a Scene

Load your scene into the app before calling `.run()`:

```rust
App::new(Config::default())
    .add_plugin(Core3D)
    .load_scene(MyScene)
    .run();
```

## Node Hierarchy

Nodes can have children, creating a parent-child hierarchy. Children are positioned relative to their parent:

```rust
scene.add(
    "Player",
    RigidBody3DBuilder::dynamic()
        .position((0.0, 5.0, 0.0))
        .add_child(
            "Body",
            Mesh3D::cube()
                .scale(Vec3::new(0.5, 1.0, 0.5))
                .build(),
        )
        .add_child(
            "Camera",
            Camera3D::builder()
                .position((0.0, 0.5, 0.0))
                .build(),
        )
        .build(),
);
```

In this example, the Camera is positioned relative to the Player's position.

## Next Steps

- Learn about [3D Rendering](3d-rendering.md) to add visual content
- Add [Behaviors](behavior.md) to make nodes interactive
- Implement [Physics](physics.md) for realistic movement
