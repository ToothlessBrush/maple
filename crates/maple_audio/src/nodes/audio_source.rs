use std::collections::VecDeque;

use kira::{
    sound::{static_sound::StaticSoundSettings, streaming::StreamingSoundSettings},
    track::SpatialTrackHandle,
};
use maple_engine::{Node, asset::AssetHandle, prelude::NodeTransform};

use crate::{asset::Audio, sound::SoundHandle};

pub use kira::Decibels;
pub use kira::Easing;
pub use kira::Panning;
pub use kira::PlaybackRate;
pub use kira::StartTime;
pub use kira::Tween;
pub use kira::Value;
pub use kira::clock::ClockId;
pub use kira::clock::ClockTime;
pub use kira::sound::EndPosition;
pub use kira::sound::PlaybackPosition;
pub use kira::sound::Region;

pub struct SoundSettings {
    pub start_time: StartTime,
    pub start_position: PlaybackPosition,
    pub loop_regions: Option<Region>,
    pub reverse: bool,
    pub volume: Value<Decibels>,
    pub playback_rate: Value<PlaybackRate>,
    pub panning: Value<Panning>,
    pub fade_in_tween: Option<Tween>,
}

impl Default for SoundSettings {
    fn default() -> Self {
        Self {
            start_time: StartTime::default(),
            start_position: PlaybackPosition::Seconds(0f64),
            loop_regions: None,
            reverse: false,
            volume: Value::Fixed(Decibels::IDENTITY),
            playback_rate: Value::Fixed(PlaybackRate(1.0)),
            panning: Value::Fixed(Panning::CENTER),
            fade_in_tween: None,
        }
    }
}

impl From<SoundSettings> for StaticSoundSettings {
    fn from(value: SoundSettings) -> Self {
        Self {
            start_time: value.start_time,
            start_position: value.start_position,
            loop_region: value.loop_regions,
            reverse: value.reverse,
            volume: value.volume,
            playback_rate: value.playback_rate,
            panning: value.panning,
            fade_in_tween: value.fade_in_tween,
        }
    }
}

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
            _ => {
                log::error!("unsupported spatial track command")
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
