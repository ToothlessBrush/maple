use crate::app::{App, Initialized};

pub trait Plugin {
    fn init(&self, app: &mut App<Initialized>);
}
