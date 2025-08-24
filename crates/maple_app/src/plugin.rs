use crate::app::{App, Running};

pub trait Plugin {
    fn init(&self, app: &mut App<Running>);
}
