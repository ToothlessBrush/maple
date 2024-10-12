# Quaturn

A simple 3D Game Engine in Rust!

**Warning: This is still very very ...very work in progress**

## Features

**3D Model Support:** load and manipulate 3D GLTF models\
**Customizable:** write your own behavior functions for a Models and Cameras\
**Write Your Own Shaders:** write your own shaders with GLSL\
**Easily Add UI's:** using egui you can easily set up a UI

## Getting Started

### Installation

1. Clone This Repo:

```bash
git clone https://github.com/ToothlessBrush/Quaturn.git
cd Quaturn
```

2. Running The Engine

```bash
cargo run
```

### Code Overview

-   Initialization

```rust
let mut engine = Engine::init("Title", WINDOW_WIDTH, WINDOW_HEIGHT);
```

-   Add a Model

```rust
engine
    .add_model("model_name", Model::new("res/path/to/model"))
    .rotate_euler_xyz(glm::Vec3::new(0.0, 0.0, -90.0))
    .define_ready(|model| {
        //runs when model is ready
        println!("(model_name) Is Ready!")
    })
    .define_ready(|model, fps_manager, input_manager| {
        //runs every frame
        if input_manager.keys.contains(&Key::W) {
            //move mode forward when W is pressed
            model.translate(glm::vec3(0.0, 1.0 * fps_manager.time_delta.as_sec_f32()));
        }
    })
```

-   Add a Camera

```rust
engine
        .add_camera(
            "camera",
            Camera3D::new(
                glm::vec3(10.0, 10.0, 10.0),
                glm::vec3(0.0, 0.0, 1.0),
                0.78539,
                WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
                0.1,
                1000.0,
            ),
        )
        .define_ready(|camera| {
            //ran before the first frame
            println!("camera ready");
        })
        .define_behavior(|camera, fps_manager, input_manager| {
            //ran every frame
            //println!("camera behavior");
        });
```

-   Add a Shader

```rust
engine.add_shader("default", Shader::new("res/shaders/default"));
```

-   Optionally add a UI with Egui

```rust
let ui = UI::init(&mut engine.window);
engine.add_ui("debug_panel", ui).define_ui(|ctx| {
    //ui to be drawn every frame
    egui::Window::new("Debug Panel").show(ctx, |ui| {
        ui.label("Hello World!");
    });
});
```

-   Finally Start the Render Loop

```rust
engine.begin()
```

## Contributing

Contributions are welcome! If you have suggestions for improvements, feel free to create a pull request or open an issue.

## License

This project is licensed under the MIT License

## Acknowledgments

-   [nalgebra-glm](https://crates.io/crates/nalgebra-glm)
-   [egui](https://crates.io/crates/egui)
-   [glfw](https://crates.io/crates/glfw)
-   [gl](https://crates.io/crates/gl)
