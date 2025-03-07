use std::default;
use std::error::Error;

pub mod scenes;
use nalgebra_glm::vec3;
use quaturn::utils::color::Color;
use scenes::{main_scene::MainScene, ui_scene::UIScene};

use quaturn::utils::config::{EngineConfig, Resolution};
use std::default::Default;

use quaturn::nodes::NodeBuilder;
use quaturn::{Engine, nodes::DirectionalLight};

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

type Err = Result<(), Box<dyn Error>>;

fn main() -> Err {
    let mut engine = Engine::init(EngineConfig {
        window_title: "Hello!".to_string(),
        resolution: Resolution {
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
        },
        ..Default::default()
    });

    engine.set_clear_color(0.0, 0.0, 0.0, 1.0);

    engine.load_scene(MainScene::build());

    engine.load_scene(UIScene::build(&engine.context.window));

    engine.context.scene.add(
        "direct_light",
        NodeBuilder::new(DirectionalLight::new(
            vec3(0.1, 1.0, 1.0),
            Color::from_8bit_rgb(255, 255, 255).into(),
            1000.0,
            3,
        ))
        .build(),
    )?;

    engine.begin()
}
