# 🍁 Maple 🍁

A 3D game engine written in Rust with a focus on simplicity and ease of use.

## Features

- **3D Rendering** - Built-in primitives, GLTF model loading, and PBR materials
- **Physics** - Integrated 3D physics with rapier for rigid body and collider nodes
- **Node-Based** - Hierarchical scene graph with parent-child relationships
- **Behavior System** - Event-driven behaviors using `.on()` handlers
- **Custom Shaders** - Write your own GLSL shaders for advanced effects

## Quick Start

```rust
use maple::prelude::*;

fn main() {
    App::new(Config::default())
        .add_plugin(Core3D)
        .add_plugin(Physics3D)
        .load_scene(MyScene)
        .run();
}

pub struct MyScene;

impl SceneBuilder for MyScene {
    fn build(&mut self) -> Scene {
        let mut scene = Scene::default();

        // Add a camera
        scene.add(
            "Camera",
            Camera3D::builder()
                .position((0.0, 5.0, -10.0))
                .orientation_vector(Vec3::new(0.0, -0.5, 1.0))
                .build(),
        );

        // Add lighting
        scene.add(
            "Sun",
            DirectionalLight::builder()
                .direction(Vec3::new(-1.0, -1.0, -1.0))
                .intensity(1.0)
                .build(),
        );

        // Add a cube
        scene.add(
            "Cube",
            Mesh3D::cube()
                .position((0.0, 1.0, 0.0))
                .material(
                    MaterialProperties::default()
                        .with_base_color_factor(Color::BLUE)
                )
                .build(),
        );

        scene
    }
}
```

## Example Images

![Shadows](https://raw.githubusercontent.com/ToothlessBrush/maple/main/images/Shadows.png)
![Model Loading](https://raw.githubusercontent.com/ToothlessBrush/maple/main/images/Model_Loading.png)
*This work is based on ["Japanese Restaurant Inakaya"](https://sketchfab.com/3d-models/japanese-restaurant-inakaya-97594e92c418491ab7f032ed2abbf596) by [MGuegan](https://sketchfab.com/MGuegan), licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).*

## Examples

Run the included examples:

```bash
# Physics demo with bouncing ball
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

# Acknowledgments

-   [nalgebra-glm](https://crates.io/crates/nalgebra-glm)
-   [egui](https://crates.io/crates/egui)
-   [glfw](https://crates.io/crates/glfw)
-   [gl](https://crates.io/crates/gl)
