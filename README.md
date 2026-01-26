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

add maple with cargo
```bash
cargo add maple
```

basic scene
```rust
use maple::prelude::*;

fn main() {
    App::new(Config::default())
        .add_plugin(Core3D)
        .add_plugin(Physics3D)
        .load_scene(MainScene)
        .run();
}

pub struct MainScene;

impl SceneBuilder for MainScene {
    fn build(&mut self, _assets: &AssetLibrary) -> Scene {
        let scene = Scene::default();

        scene
            .spawn(
                "Camera",
                Camera3D::builder()
                    .position((-10.0, 10.0, 10.0))
                    .far_plane(100.0)
                    .orientation_vector(
                        Vec3::ZERO
                            - Vec3 {
                                x: -10.0,
                                y: 10.0,
                                z: 10.0,
                            },
                    )
                    .build(),
            )
            .on::<Ready>(|ctx| {
                ctx.game.get_resource_mut::<Input>().set_cursor_locked(true);
            })
            .on::<Update>(Camera3D::free_fly(1.0, 1.0));

        scene.spawn(
            "floor",
            Mesh3D::plane()
                .position((0.0, -2.0, 0.0))
                .scale_factor(10.0)
                .build(),
        );

        scene.spawn(
            "cube",
            Mesh3D::cube()
                .material(MaterialProperties::default().with_base_color_factor(Color::BLUE))
                .build(),
        );

        scene.spawn(
            "direct",
            DirectionalLight::builder()
                .direction((-1.0, -1.0, -1.0))
                .intensity(10.0)
                .bias(0.0001)
                .build(),
        );

        scene
    }
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
