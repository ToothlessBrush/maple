use crate::app::{App, Init, Running};

pub trait Plugin {
    /// Called during App<Init> phase, before .run()
    /// Use this to initialize resources that don't need the renderer
    #[allow(unused)]
    fn setup(&self, app: &mut App<Init>) {}

    /// Called when the app is ready and the renderer is initialized
    #[allow(unused)]
    fn ready(&self, app: &mut App<Running>) {}

    /// Called every frame
    #[allow(unused)]
    fn update(&self, app: &mut App<Running>) {}

    /// Called every tick or 1/60
    #[allow(unused)]
    fn fixed_update(&self, app: &mut App<Running>) {}
}
