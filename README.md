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
```rust,ignore
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

```rust,ignore 
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

```rust,ignore 
use maple::{Engine, config::{EngineConfig, Resolution}};
use std::{default::Default, error::Error};

// create and import the main scene module
pub mod main_scene;
use main_scene::MainScene;

fn main() -> Result<(), Box<dyn Error>> {
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

```rust,ignore 
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

```rust,ignore 
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
nning local AI models both on a 
Your scene file should now look like this:

```rust,ignore
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
}
```

### Lighting

If everything has gone well then when you run the program you can see a window pop up and a pyramid in the center of the screen. However, the mesh is really dark since its just being lit by ambient light so now adding a light and some ground to the scene should help.

```rust,ignore
use maple::{
    context::scene::Scene,
    math,
    nodes::{Buildable, Builder, DirectionalLight, Model, model::Primitive},
};

scene.add(
    "light",
    DirectionalLight::builder()
        .direction(math::vec3(-1.0, 1.0, 0.0))
        .build(),
);

scene.add(
    "ground",
    Model::builder()
        .add_primitive(Primitive::Plane)
        .position(math::vec3(0.0, -2.0, 0.0))
        .scale_factor(10.0)
        .build(),
);
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
