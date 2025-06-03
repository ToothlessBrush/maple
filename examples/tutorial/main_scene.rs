use maple::{
    components::Event,
    context::scene::Scene,
    nodes::{Buildable, Builder, Empty},
};

use std::io::{self, Write};

pub struct MainScene;

impl MainScene {
    pub fn build() -> Scene {
        let mut scene = Scene::default();

        scene.add(
            "fps",
            Empty::builder()
                .on(Event::Update, |_node, ctx| {
                    println!("{}", ctx.frame.frame_info);
                    let mut stdout = io::stdout();
                    write!(stdout, "\x1b[10A").unwrap(); // Move up 7 lines
                    stdout.flush().unwrap();
                })
                .build(),
        );

        scene
    }
}
