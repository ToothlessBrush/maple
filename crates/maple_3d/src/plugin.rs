use maple_app::Plugin;

pub struct Core3D {}

impl Plugin for Core3D {
    fn init(&self, app: &mut maple::prelude::App<maple::app::app::Running>) {}
    fn update(&self, app: &mut maple::prelude::App<maple::app::app::Running>) {}
}
