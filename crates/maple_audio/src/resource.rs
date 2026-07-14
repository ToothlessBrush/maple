use glam::{Quat, Vec3};
use kira::{AudioManager as Manager, listener::ListenerHandle};
use maple_engine::prelude::Resource;

pub struct AudioManager {
    pub(crate) manager: Manager,
    pub(crate) listener: ListenerHandle,
}

impl AudioManager {
    pub(crate) fn new(mut manager: Manager) -> Self {
        let listener = manager
            .add_listener(Vec3::default(), Quat::default())
            .expect("listener to be created there should only be one listener");
        Self { manager, listener }
    }
}

impl Resource for AudioManager {}
