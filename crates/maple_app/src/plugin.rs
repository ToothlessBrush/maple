use crate::app::{App, Running};

pub trait Plugin {
    /// called when the app is ready to load plugins
    #[allow(unused)]
    fn init(&self, app: &mut App<Running>) {}
    /// called every frame
    #[allow(unused)]
    fn update(&self, app: &mut App<Running>) {}
    /// called every tick or 1/60
    #[allow(unused)]
    fn fixed_update(&self, app: &mut App<Running>) {}
}
