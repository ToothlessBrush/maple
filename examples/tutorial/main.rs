use maple::prelude::*;

// create and import the main scene module
pub mod main_scene;
use main_scene::MainScene;

fn main() {
    App::new(Config::default()).load_scene(MainScene).run()
}
