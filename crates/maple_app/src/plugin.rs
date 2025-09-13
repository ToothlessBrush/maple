use crate::app::{App, Running};

pub trait Plugin {
    /// called when the app is ready to load plugins
    fn init(&self, _app: &mut App<Running>) {}
    /// called every frame
    fn update(&self, _app: &mut App<Running>) {}
}
