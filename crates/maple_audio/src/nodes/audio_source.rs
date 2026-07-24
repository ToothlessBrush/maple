use std::collections::VecDeque;

use kira::{Decibels, StartTime, Tween, Value, track::SpatialTrackHandle};
use maple_engine::{Node, asset::AssetHandle, prelude::NodeTransform};

use crate::{asset::Audio, settings::SoundSettings, sound::SoundHandle};

pub enum DeferredSourceCommand {
    SetVolume {
        volume: Value<Decibels>,
        tween: Tween,
    },
    Pause(Tween),
    Resume(Tween),
    ResumeAt {
        start_time: StartTime,
        tween: Tween,
    },
}

pub enum SourceHandle {
    SpatialHandle(SpatialTrackHandle),
    DeferredCommands(VecDeque<DeferredSourceCommand>),
}

impl Default for SourceHandle {
    fn default() -> Self {
        Self::DeferredCommands(VecDeque::new())
    }
}

impl SourceHandle {
    pub fn apply_command_spatial(handle: &mut SpatialTrackHandle, cmd: DeferredSourceCommand) {
        match cmd {
            DeferredSourceCommand::SetVolume { volume, tween } => handle.set_volume(volume, tween),
            DeferredSourceCommand::Pause(tween) => handle.pause(tween),
            DeferredSourceCommand::Resume(tween) => handle.resume(tween),
            DeferredSourceCommand::ResumeAt { start_time, tween } => {
                handle.resume_at(start_time, tween)
            }
        }
    }

    pub fn apply_commands_spatial(
        handle: &mut SpatialTrackHandle,
        cmds: &mut VecDeque<DeferredSourceCommand>,
    ) {
        while let Some(command) = cmds.pop_front() {
            Self::apply_command_spatial(handle, command);
        }
    }
}

#[derive(Default)]
pub struct AudioSource {
    pub transform: NodeTransform,
    pub(crate) handle: SourceHandle,
    pub(crate) queue: VecDeque<(AssetHandle<Audio>, SoundSettings, SoundHandle)>,
}

impl Node for AudioSource {
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }
}

impl AudioSource {
    pub fn play(&mut self, source: AssetHandle<Audio>, settings: SoundSettings) -> SoundHandle {
        let handle = SoundHandle::default();
        self.queue.push_back((source, settings, handle.clone()));
        handle
    }

    pub fn set_volume(&mut self, volume: Decibels, tween: Tween) {
        match &mut self.handle {
            SourceHandle::SpatialHandle(handle) => handle.set_volume(volume, tween),
            SourceHandle::DeferredCommands(commands) => {
                commands.push_back(DeferredSourceCommand::SetVolume {
                    volume: volume.into(),
                    tween,
                })
            }
        }
    }

    pub fn pause(&mut self, tween: Tween) {
        match &mut self.handle {
            SourceHandle::SpatialHandle(handle) => handle.pause(tween),
            SourceHandle::DeferredCommands(commands) => {
                commands.push_back(DeferredSourceCommand::Pause(tween))
            }
        }
    }

    pub fn resume(&mut self, tween: Tween) {
        match &mut self.handle {
            SourceHandle::SpatialHandle(handle) => handle.resume(tween),
            SourceHandle::DeferredCommands(commands) => {
                commands.push_back(DeferredSourceCommand::Resume(tween))
            }
        }
    }
}
