use std::error::Error;

pub mod scenes;
use scenes::{main_scene::MainScene, ui_scene::UIScene};

use quaturn::Engine;

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;

fn main() -> Result<(), Box<dyn Error>> {
    let mut engine = Engine::init("Hello Pyramid", WINDOW_WIDTH, WINDOW_HEIGHT);

    engine.set_clear_color(0.0, 0.0, 0.0, 1.0);

    engine.load_scene(MainScene::build());

    engine.load_scene(UIScene::build(&engine.context.window));

    engine.begin()
}
