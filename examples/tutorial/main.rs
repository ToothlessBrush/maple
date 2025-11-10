use maple::prelude::*;

// create and import the main scene module
pub mod main_scene;
use main_scene::MainScene;
use maple_3d::plugin::Core3D;

fn main() {
    App::new(Config::default())
        .add_plugin(Core3D)
        .load_scene(MainScene)
        .run();
}
