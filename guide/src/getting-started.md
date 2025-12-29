# Getting Started with Maple

This guide will walk you through creating your first Maple application.

## Installation

Add Maple to your `Cargo.toml`:

```toml
[dependencies]
maple = "0.1"  # Check for the latest version
```

## Creating Your First App

A minimal Maple application looks like this:

```rust
use maple::prelude::*;

fn main() {
    App::new(Config::default())
        .add_plugin(Core3D)
        .load_scene(MyScene)
        .run();
}

pub struct MyScene;

impl SceneBuilder for MyScene {
    fn build(&mut self) -> Scene {
        let mut scene = Scene::default();

        // Add nodes to your scene here

        scene
    }
}
```

## Configuration

You can customize the app configuration:

```rust
App::new(Config {
    window_mode: WindowMode::FullScreen,
    ..Default::default()
})
```

Available configuration options include:
- `window_mode` - Set windowed, fullscreen, or borderless mode
- Window resolution and title
- And more...

## Plugins

Plugins add functionality to your app. Common plugins include:

- **`Core3D`** - Enables 3D rendering capabilities
- **`Physics3D`** - Adds physics simulation

Add plugins with `.add_plugin()`:

```rust
App::new(Config::default())
    .add_plugin(Core3D)
    .add_plugin(Physics3D)
    .load_scene(MyScene)
    .run();
```

## Running Your App

Once you've configured your app and loaded a scene, call `.run()` to start the engine:

```rust
app.run();
```

This will open the window and begin the game loop.

## Next Steps

- Learn about [Scene Creation](scenes.md)
- Explore [3D Rendering](3d-rendering.md)
- Add [Behaviors](behavior.md) to your nodes
- Implement [Physics](physics.md)
