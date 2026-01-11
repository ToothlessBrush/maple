use maple_engine::{
    input::Input,
    prelude::{FixedUpdate, Frame, Ready, Update},
};

use crate::Plugin;

pub struct DefaultPlugin;

impl Plugin for DefaultPlugin {
    fn setup(&self, app: &mut crate::App<crate::Init>) {
        app.context_mut().insert_resource(Frame::default());
    }

    fn ready(&self, app: &mut crate::App<crate::Running>) {
        let window = app.window().clone();
        app.context_mut().insert_resource(Input::new(window));

        // sync world positions before ready (since they are synced after between update and
        // render normally)
        app.context().scene.sync_world_transform();

        app.context().emit(Ready);
    }

    fn update(&self, app: &mut crate::App<crate::Running>) {
        let dt = app.context().get_resource::<Frame>().time_delta_f32;
        app.context().emit(Update { dt });
    }

    fn fixed_update(&self, app: &mut crate::App<crate::Running>) {
        app.context().emit(FixedUpdate);
    }
}
