use maple::prelude::*;

pub mod player_scene;
use maple_3d::plugin::Core3D;
use maple_physics::plugin::Physics3D;
use player_scene::PlayerScene;

fn main() {
    App::new(Config {
        window_mode: WindowMode::FullScreen,
        ..Default::default()
    })
    .add_plugin(Core3D)
    .add_plugin(Physics3D)
    .load_scene(PlayerScene)
    .run();
}
