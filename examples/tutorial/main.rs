use maple::Engine;
use maple::utils::config::EngineConfig;
use std::default::Default;
use std::error::Error;

// create and import the main scene module
pub mod main_scene;
use main_scene::MainScene;

fn main() -> Result<(), Box<dyn Error>> {
    let mut engine = Engine::init(EngineConfig {
        window_title: "Hello, Window!".to_string(),
        window_mode: maple::utils::config::WindowMode::Windowed,
        ..Default::default()
    })?;

    // load the scene into the engine
    engine.load_scene(MainScene::build());

    engine.begin()
}
