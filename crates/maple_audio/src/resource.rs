use std::collections::VecDeque;

use glam::{Quat, Vec3};
use kira::{AudioManager as Manager, listener::ListenerHandle};
use maple_engine::{asset::AssetHandle, prelude::Resource};

use crate::{asset::Audio, nodes::audio_source::SoundSettings};

pub struct AudioManager {
    pub(crate) manager: Manager,
    pub(crate) listener: Option<ListenerHandle>,
    pub(crate) queue: VecDeque<(AssetHandle<Audio>, SoundSettings)>,
}

impl AudioManager {
    pub(crate) fn new(manager: Manager) -> Self {
        Self {
            manager,
            listener: None,
            queue: VecDeque::default(),
        }
    }

    pub fn play(&mut self, sound: AssetHandle<Audio>, settings: SoundSettings) {
        self.queue.push_back((sound, settings));
    }
}

impl Resource for AudioManager {}
