use maple_app::Plugin;

use crate::render_passes::{self, main_pass::MainPass};

pub struct Core3D {}

impl Plugin for Core3D {
    fn init(&self, app: &mut maple_app::App<maple_app::Running>) {
        let mut graph = app.renderer_mut().graph();

        graph.add_node(render_passes::MAIN_PASS, MainPass::default());
    }
}
