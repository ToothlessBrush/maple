use maple_3d::plugin::Core3D;
use maple_app::{App, Init, Plugin};
use maple_physics::plugin::Physics3D;

pub struct DefaultPlugins;

impl Plugin for DefaultPlugins {
    fn setup(&self, app: maple_app::prelude::App<maple_app::prelude::Init>) -> App<Init> {
        let app = app.add_plugin(Core3D).add_plugin(Physics3D);

        app
    }
}
