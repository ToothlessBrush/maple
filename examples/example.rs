use std::default;
use std::error::Error;

pub mod scenes;
use scenes::{main_scene::MainScene, ui_scene::UIScene};

use quaturn::utils::config::EngineConfig;
use std::default::Default;

use quaturn::nodes::NodeBuilder;
use quaturn::{Engine, nodes::DirectionalLight};

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;

fn main() -> Result<(), Box<dyn Error>> {
    let mut engine = Engine::init(EngineConfig {
        window_title: "Hello!".to_string(),
        ..Default::default()
    });

    engine.set_clear_color(0.0, 0.0, 0.0, 1.0);

    engine.load_scene(MainScene::build());

    engine.load_scene(UIScene::build(&engine.context.window));

    engine.context.scene.add(
        "direct_light",
        NodeBuilder::new(DirectionalLight::new(100.0, 3, &[0.25, 0.5])).build(),
    );

    engine.begin()
}
