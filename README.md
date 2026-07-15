# 🍁 Maple 🍁

A 3D game engine written in Rust with a focus on simplicity and ease of use.

## Features

- **3D Rendering** - Built-in primitives, GLTF model loading, and PBR materials
- **Physics** - Integrated 3D physics with rapier for rigid body and collider nodes
- **Node-Based** - Hierarchical scene graph with parent-child relationships
- **Behavior System** - Event-driven behaviors using `.on()` handlers

## Showcase

![Bistro](https://raw.githubusercontent.com/ToothlessBrush/maple/main/images/bistro.png)
![Helmet](https://raw.githubusercontent.com/ToothlessBrush/maple/main/images/helmet.png)

## Quick Start

Add Maple with Cargo:
```bash
cargo add maple
```

Create a basic scene:
```rust
pub use maple::prelude::*;

fn main() {
    App::default().add_plugin(Core3D).load_scene(scene).run()
}

fn scene(assets: &AssetLibrary) -> Scene {
    let scene = Scene::default();

    scene
        .spawn(Empty::default())
        .on::<FixedUpdate>(|ctx| {
            ctx.node_mut().transform.rotate((0.1, 1.0, 0.1), 0.1);
        })
        .spawn_child(DirectionalLight::builder().direction((1.0, -1.0, -1.0)));

    scene.spawn(
        Camera3D::builder()
            .position((2.0, 2.0, 2.0))
            .looking_at(Vec3::ZERO),
    );

    scene.spawn(
        MeshInstance3D::builder()
            .mesh(assets.add(Plane::default().size((2.0, 2.0))))
            .material(assets.add(Color::WHITE)),
    );

    scene.spawn(
        MeshInstance3D::builder()
            .mesh(assets.add(Cuboid::default().half_extent(0.1)))
            .material(assets.add(Color::BLUE))
            .position((0.0, 0.3, 0.0)),
    );

    scene
}
```

## Examples

Check out the examples by cloning this repo

```bash
# Physics demo
cargo run --example physics_demo

# First-person player controller
cargo run --example player_controller

# Model loading from GLTF
cargo run --example model_loading
```

# Contributing

Contributions are welcome! If you have suggestions for improvements, feel free to create a pull request or open an issue.

# License

This project is licensed under the MIT License
