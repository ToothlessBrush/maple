# 🍁 Maple 🍁

A simple 3D Game Engine in Rust!

## Features

**3D Model Support:** load and manipulate 3D GLTF models\
**Customizable:** create your own Nodes and use predefined nodes for more specific functionality\
**Write Your Own Shaders:** write your own shaders with GLSL\
**Easily Add UI's:** using egui you can easily set up a UI

## Example Images

![Shadows](https://raw.githubusercontent.com/ToothlessBrush/maple/main/images/Shadows.png)
![Model Loading](https://raw.githubusercontent.com/ToothlessBrush/maple/main/images/Model_Loading.png)
This work is based on ["Japanese Restaurant Inakaya"](https://sketchfab.com/3d-models/japanese-restaurant-inakaya-97594e92c418491ab7f032ed2abbf596) by [MGuegan](https://sketchfab.com/MGuegan), licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).

# Guide to using the Engine

this guide goes over the basics of initializing the engine, adding nodes, and defining custom nodes.

you can find the code used in this tutorial [here](https://github.com/ToothlessBrush/maple/tree/main/examples/tutorial).

## Initialization

lets start out by creating the bare minimum code to create and start the engine.
```rust
use maple::{Engine, config::{EngineConfig, Resolution}};
use std::{default::Default, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    let mut engine = Engine::init(EngineConfig {
        window_title: "Hello, Window!".to_string(),
        resolution: Resolution {
            width: 1920,
            height: 1080,
        },
        ..Default::default()
    })?;

    engine.begin()
}
```

if you run this code it creates a window at 1920x1080 resolution, but its a bit empty so next we'll create a scene.

## Scene Creation

in this section we'll create the engine's scene. scenes can be defined then added to the engine to render that scene.

to keep it as organized, we should define the scene in a seperate file then import it into main.

```rust
use maple::context::scene::Scene;

pub struct MainScene;

impl MainScene {
    pub fn build() -> Scene {
        let mut scene = Scene::default();
        
        /* Scene will go here */

        scene
    }
}
```

this code creates a function that will build the scene when called

before we add Nodes to the scene lets load this scene into the engine

```rust 
use maple::{Engine, config::EngineConfig};
use std::{default::Default, error::Error};

// create and import the main scene module
pub mod main_scene;
use main_scene::MainScene;

fn main() {
    let mut engine = Engine::init(EngineConfig {
        window_title: "Hello, Window!".to_string(),
        resolution: Resolution {
            width: 1920,
            height: 1080,
        },
        ..Default::default()
    })?;

    // load the scene into the engine
    engine.load_scene(MainScene::build());

    engine.begin()
}
```

## Node Creation

now we can finally add nodes to the Scene!

Nodes are easiest to build with the NodeBuilder which is a struct that helps define nodes. we'll use this to create a model and camera.

### Model

lets create a pyramid model.

```rust 
use maple::context::scene::Scene;
use maple::nodes::{model::Primitive, Model, ModelBuilder, Builder, Buildable};

scene
    .add(
        "pyramid", // name
        Model::builder()
            .add_primitive(Primitive::Pyramid) // add a Pyramid mesh to the model node
            .build(), // builds the node
    );
```
### Camera3D

thats great but the engine doesnt know where to render the pyramid from for that we need a camera. cameras define the perspective so we'll need to position it properly.

```rust 
use maple::{
    math,
    nodes::{Buildable, Builder, Camera3D},
};

scene.add(
    "camera",
    Camera3D::builder()
        .position(math::vec3(0.0, 5.0, -10.0))
        .orientation_vector(math::vec3(0.0, -0.5, 1.0))
        .build(),
);

```

Your scene file should now look like this:

```rust 
use maple::{
    context::scene::Scene,
    math,
    nodes::{Buildable, Builder, Camera3D, Model, model::Primitive},
};

pub struct MainScene;

impl MainScene {
    pub fn build() -> Scene {
        let mut scene = Scene::default();

        scene.add(
            "camera",
            Camera3D::builder()
                .position(math::vec3(0.0, 5.0, -10.0))
                .orientation_vector(math::vec3(0.0, -0.5, 1.0))
                .build(),
        );

        scene.add(
            "pyramid",
            Model::builder().add_primitive(Primitive::Pyramid).build(),
        );

        scene
    }
}```




## Shader Uniforms

for building your own shaders the engine applies these uniforms you can also define your own uniforms with

```rust
shader.set_uniform(name, value)
```

| Uniform Name             | Type        | Description                                                    |
| ------------------------ | ----------- | -------------------------------------------------------------- |
| `diffuse0`               | `sampler2D` | Diffuse texture sampler                                        |
| `specular0`              | `sampler2D` | Specular texture sampler                                       |
| `shadowMap`              | `sampler2D` | Shadow map texture sampler                                     |
| `baseColorFactor`        | `vec4`      | Base color factor for the material (RGBA)                      |
| `useTexture`             | `bool`      | Whether to use the texture for the object                      |
| `useAlphaCutoff`         | `bool`      | Whether alpha cutoff is applied                                |
| `alphaCutoff`            | `float`     | Alpha cutoff value for transparency                            |
| `lightColor`             | `vec4`      | Color of the light (RGBA)                                      |
| `lightPos`               | `vec3`      | Position of the light source in world space                    |
| `camPos`                 | `vec3`      | Camera position in world space                                 |
| `u_directLightDirection` | `vec3`      | Direction of the directional light (normalized vector)         |
| `u_SpecularStrength`     | `float`     | Strength of the specular highlights                            |
| `u_AmbientStrength`      | `float`     | Strength of the ambient lighting                               |
| `u_bias`                 | `float`     | Bias value for shadow mapping to avoid shadow acne             |
| `u_BackgroundColor`      | `vec3`      | Background color of the scene (RGB)                            |
| `u_VP`                   | `mat4`      | View projection matrix (combined model-view-projection matrix) |
| `u_Model`                | `mat4`      | Model matrix for the object                                    |
| `u_lightSpaceMatrix`     | `mat4`      | Light space matrix for shadow mapping                          |

# Contributing

Contributions are welcome! If you have suggestions for improvements, feel free to create a pull request or open an issue.

# License

This project is licensed under the MIT License

# Acknowledgments

-   [nalgebra-glm](https://crates.io/crates/nalgebra-glm)
-   [egui](https://crates.io/crates/egui)
-   [glfw](https://crates.io/crates/glfw)
-   [gl](https://crates.io/crates/gl)
