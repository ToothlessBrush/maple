use std::collections::VecDeque;

use kira::{AudioManager as Manager, listener::ListenerHandle};
use maple_engine::{asset::AssetHandle, prelude::Resource};

use crate::{asset::Audio, settings::SoundSettings, sound::SoundHandle};

pub struct AudioManager {
    pub(crate) manager: Manager,
    pub(crate) listener: Option<ListenerHandle>,
    pub(crate) queue: VecDeque<(AssetHandle<Audio>, SoundSettings, SoundHandle)>,
}

impl AudioManager {
    pub(crate) fn new(manager: Manager) -> Self {
        Self {
            manager,
            listener: None,
            queue: VecDeque::default(),
        }
    }

    pub fn play(&mut self, sound: AssetHandle<Audio>, settings: SoundSettings) -> SoundHandle {
        let handle = SoundHandle::default();
        self.queue.push_back((sound, settings, handle.clone()));
        handle
    }
}

impl Resource for AudioManager {}
