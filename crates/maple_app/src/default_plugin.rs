use maple_engine::{
    prelude::{FixedUpdate, Frame, Update},
    resources::Input,
};

use crate::Plugin;

pub struct DefaultPlugin;

impl Plugin for DefaultPlugin {
    fn setup(&self, app: &mut crate::App<crate::Init>) {
        match env_logger::Builder::from_env(
            env_logger::Env::default().default_filter_or("info,wgpu_hal=warn,naga=warn"),
        )
        .try_init()
        {
            Ok(_) => {}
            Err(e) => log::info!("Ignoring Logger: {e}"),
        }

        app.context_mut()
            .assets
            .register_loader(maple_renderer::texture_asset::TextureAssetLoader);
    }

    fn ready(&self, app: &mut crate::App<crate::Running>) {
        let window = app.window().clone();
        app.context_mut().insert_resource(Frame::default());
        app.context_mut().insert_resource(Input::new(window));

        // sync world positions before ready (since they are synced after between update and
        // render normally)
        app.context().scene.sync_world_transform();
    }

    fn update(&self, app: &mut crate::App<crate::Running>) {
        let dt = app.context().get_resource::<Frame>().time_delta_f32;
        app.context().pop_ready_queue();
        app.context().emit(Update { dt });
    }

    fn fixed_update(&self, app: &mut crate::App<crate::Running>) {
        app.context().emit(FixedUpdate);
    }
}
