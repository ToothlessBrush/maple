use std::default;
use std::error::Error;

pub mod scenes;
use gl::RED;
use nalgebra_glm::vec3;
use quaturn::nodes::directional_light::DirectionalLightBuilder;
use quaturn::utils::color::{Color, BLACK, CYAN, GREEN, MAGENTA, WHITE, YELLOW};
use scenes::{main_scene::MainScene, ui_scene::UIScene};

use quaturn::utils::config::{EngineConfig, Resolution};
use std::default::Default;

use quaturn::nodes::NodeBuilder;
use quaturn::{nodes::DirectionalLight, Engine};

const WINDOW_WIDTH: u32 = 1920;
const WINDOW_HEIGHT: u32 = 1080;

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

    engine.set_clear_color(Color::from_8bit_rgb(50, 50, 50));

    engine.load_scene(MainScene::build());

    engine.load_scene(UIScene::build(&engine.context.window));

    engine.context.scene.add(
        "direct_light",
        NodeBuilder::<DirectionalLight>::create(
            vec3(0.1, 0.9, 0.5),
            Color::from_8bit_rgb(255, 255, 255).into(),
        )
        .build(),
    )?;

    //engine.context.scene.add(
    //    "direct_light2",
    //    NodeBuilder::<DirectionalLight>::create(
    //        vec3(-0.1, -0.9, -0.5),
    //        Color::from_8bit_rgb(255, 255, 255).into(),
    //    )
    //    .build(),
    //)?;

    engine.begin()
}
