use quaturn::utils::config::EngineConfig;
use quaturn::Engine;
use std::default::Default;

// create and import the main scene module
pub mod main_scene;
use main_scene::MainScene;

fn main() {
    let mut engine = Engine::init(EngineConfig {
        window_title: "Hello, Window!".to_string(),
        window_mode: quaturn::utils::config::WindowMode::Windowed,
        ..Default::default()
    });

    // load the scene into the engine
    engine.load_scene(MainScene::build(&engine.context.window));

    engine.begin();
}
