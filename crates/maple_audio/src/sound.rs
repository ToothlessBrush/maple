use std::{collections::VecDeque, ops::DerefMut, sync::Arc};

use kira::{
    Decibels, Panning, PlaybackRate, StartTime, Tween, Value,
    sound::{
        FromFileError, Region, static_sound::StaticSoundHandle, streaming::StreamingSoundHandle,
    },
};
use parking_lot::Mutex;

pub use kira::sound::IntoOptionalRegion;

pub enum DeferredSoundCommand {
    SetVolume {
        volume: Value<Decibels>,
        tween: Tween,
    },
    SetPlaybackRate {
        playback_rate: Value<PlaybackRate>,
        tween: Tween,
    },
    SetPanning {
        panning: Value<Panning>,
        tween: Tween,
    },
    SetLoopReigon(Option<Region>),
    Pause(Tween),
    Resume(Tween),
    ResumeAt {
        start_time: StartTime,
        tween: Tween,
    },
    Stop(Tween),
}

impl DeferredSoundCommand {
    fn apply_command(handle: &mut StaticSoundHandle, cmd: DeferredSoundCommand) {
        match cmd {
            DeferredSoundCommand::SetVolume { volume, tween } => handle.set_volume(volume, tween),
            DeferredSoundCommand::SetPlaybackRate {
                playback_rate,
                tween,
            } => handle.set_playback_rate(playback_rate, tween),
            DeferredSoundCommand::SetPanning { panning, tween } => {
                handle.set_panning(panning, tween)
            }
            DeferredSoundCommand::SetLoopReigon(region) => handle.set_loop_region(region),
            DeferredSoundCommand::Pause(tween) => handle.pause(tween),
            DeferredSoundCommand::Resume(tween) => handle.pause(tween),
            DeferredSoundCommand::ResumeAt { start_time, tween } => {
                handle.resume_at(start_time, tween)
            }
            DeferredSoundCommand::Stop(tween) => handle.stop(tween),
        }
    }

    pub fn apply_commands(
        handle: &mut StaticSoundHandle,
        cmds: &mut VecDeque<DeferredSoundCommand>,
    ) {
        while let Some(cmd) = cmds.pop_front() {
            Self::apply_command(handle, cmd);
        }
    }

    fn apply_command_streaming(
        handle: &mut StreamingSoundHandle<FromFileError>,
        cmd: DeferredSoundCommand,
    ) {
        match cmd {
            DeferredSoundCommand::SetVolume { volume, tween } => handle.set_volume(volume, tween),
            DeferredSoundCommand::SetPlaybackRate {
                playback_rate,
                tween,
            } => handle.set_playback_rate(playback_rate, tween),
            DeferredSoundCommand::SetPanning { panning, tween } => {
                handle.set_panning(panning, tween)
            }
            DeferredSoundCommand::SetLoopReigon(region) => handle.set_loop_region(region),
            DeferredSoundCommand::Pause(tween) => handle.pause(tween),
            DeferredSoundCommand::Resume(tween) => handle.pause(tween),
            DeferredSoundCommand::ResumeAt { start_time, tween } => {
                handle.resume_at(start_time, tween)
            }
            DeferredSoundCommand::Stop(tween) => handle.stop(tween),
        }
    }

    pub fn apply_commands_streaming(
        handle: &mut StreamingSoundHandle<FromFileError>,
        cmds: &mut VecDeque<DeferredSoundCommand>,
    ) {
        while let Some(cmd) = cmds.pop_front() {
            Self::apply_command_streaming(handle, cmd);
        }
    }
}

pub enum SoundState {
    Handle(StaticSoundHandle),
    StreamingHandle(StreamingSoundHandle<FromFileError>),
    Deferred(VecDeque<DeferredSoundCommand>),
}

impl Default for SoundState {
    fn default() -> Self {
        Self::Deferred(VecDeque::default())
    }
}

#[derive(Default, Clone)]
pub struct SoundHandle(pub(crate) Arc<Mutex<SoundState>>);

impl SoundHandle {
    pub fn set_volume(&mut self, volume: impl Into<Value<Decibels>>, tween: Tween) {
        let mut state = self.0.lock();
        match state.deref_mut() {
            SoundState::Handle(handle) => handle.set_volume(volume, tween),
            SoundState::StreamingHandle(handle) => handle.set_volume(volume, tween),
            SoundState::Deferred(commands) => commands.push_back(DeferredSoundCommand::SetVolume {
                volume: volume.into(),
                tween,
            }),
        }
    }

    pub fn set_playback_rate(
        &mut self,
        playback_rate: impl Into<Value<PlaybackRate>>,
        tween: Tween,
    ) {
        let mut state = self.0.lock();
        match state.deref_mut() {
            SoundState::Handle(handle) => handle.set_playback_rate(playback_rate, tween),
            SoundState::StreamingHandle(handle) => handle.set_playback_rate(playback_rate, tween),
            SoundState::Deferred(commands) => {
                commands.push_back(DeferredSoundCommand::SetPlaybackRate {
                    playback_rate: playback_rate.into(),
                    tween,
                })
            }
        }
    }

    pub fn set_panning(&mut self, panning: impl Into<Value<Panning>>, tween: Tween) {
        let mut state = self.0.lock();
        match state.deref_mut() {
            SoundState::Handle(handle) => handle.set_panning(panning, tween),
            SoundState::StreamingHandle(handle) => handle.set_panning(panning, tween),
            SoundState::Deferred(commands) => {
                commands.push_back(DeferredSoundCommand::SetPanning {
                    panning: panning.into(),
                    tween,
                })
            }
        }
    }

    pub fn set_loop_region(&mut self, region: impl IntoOptionalRegion) {
        let mut state = self.0.lock();
        match state.deref_mut() {
            SoundState::Handle(handle) => handle.set_loop_region(region),
            SoundState::StreamingHandle(handle) => handle.set_loop_region(region),
            SoundState::Deferred(commands) => commands.push_back(
                DeferredSoundCommand::SetLoopReigon(region.into_optional_region()),
            ),
        }
    }

    pub fn pause(&mut self, tween: Tween) {
        let mut state = self.0.lock();
        match state.deref_mut() {
            SoundState::Handle(handle) => handle.pause(tween),
            SoundState::StreamingHandle(handle) => handle.pause(tween),
            SoundState::Deferred(commands) => {
                commands.push_back(DeferredSoundCommand::Pause(tween))
            }
        }
    }

    pub fn resume(&mut self, tween: Tween) {
        let mut state = self.0.lock();
        match state.deref_mut() {
            SoundState::Handle(handle) => handle.resume(tween),
            SoundState::StreamingHandle(handle) => handle.resume(tween),
            SoundState::Deferred(commands) => {
                commands.push_back(DeferredSoundCommand::Resume(tween))
            }
        }
    }

    pub fn resume_at(&mut self, start_time: StartTime, tween: Tween) {
        let mut state = self.0.lock();
        match state.deref_mut() {
            SoundState::Handle(handle) => handle.resume_at(start_time, tween),
            SoundState::StreamingHandle(handle) => handle.resume_at(start_time, tween),
            SoundState::Deferred(commands) => {
                commands.push_back(DeferredSoundCommand::ResumeAt { start_time, tween })
            }
        }
    }

    pub fn stop(&mut self, tween: Tween) {
        let mut state = self.0.lock();
        match state.deref_mut() {
            SoundState::Handle(handle) => handle.stop(tween),
            SoundState::StreamingHandle(handle) => handle.stop(tween),
            SoundState::Deferred(commands) => commands.push_back(DeferredSoundCommand::Stop(tween)),
        }
    }
}
