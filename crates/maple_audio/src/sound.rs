use std::{collections::VecDeque, ops::DerefMut, sync::Arc};

#[cfg(not(target_arch = "wasm32"))]
use kira::sound::streaming::StreamingSoundHandle;
use kira::{
    Decibels, Panning, PlaybackRate, StartTime, Tween, Value,
    sound::{FromFileError, Region, static_sound::StaticSoundHandle},
};

use parking_lot::Mutex;

pub use kira::sound::IntoOptionalRegion;

pub(crate) enum DeferredSoundCommand {
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

    #[cfg(not(target_arch = "wasm32"))]
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

    #[cfg(not(target_arch = "wasm32"))]
    pub fn apply_commands_streaming(
        handle: &mut StreamingSoundHandle<FromFileError>,
        cmds: &mut VecDeque<DeferredSoundCommand>,
    ) {
        while let Some(cmd) = cmds.pop_front() {
            Self::apply_command_streaming(handle, cmd);
        }
    }
}

pub(crate) enum SoundState {
    Handle(StaticSoundHandle),
    #[cfg(not(target_arch = "wasm32"))]
    StreamingHandle(StreamingSoundHandle<FromFileError>),
    Deferred(VecDeque<DeferredSoundCommand>),
}

impl Default for SoundState {
    fn default() -> Self {
        Self::Deferred(VecDeque::default())
    }
}

/// Provides a handle to the played sound allowing changing the configuration
#[derive(Default, Clone)]
pub struct SoundHandle(pub(crate) Arc<Mutex<SoundState>>);

impl SoundHandle {
    /// changes the volume of the sound over a [`Tween`]
    pub fn set_volume(&mut self, volume: impl Into<Value<Decibels>>, tween: Tween) {
        let mut state = self.0.lock();
        match state.deref_mut() {
            SoundState::Handle(handle) => handle.set_volume(volume, tween),
            #[cfg(not(target_arch = "wasm32"))]
            SoundState::StreamingHandle(handle) => handle.set_volume(volume, tween),
            SoundState::Deferred(commands) => commands.push_back(DeferredSoundCommand::SetVolume {
                volume: volume.into(),
                tween,
            }),
        }
    }

    /// changes the playback rate or how fast the sound plays of the sound over a [`Tween`]
    pub fn set_playback_rate(
        &mut self,
        playback_rate: impl Into<Value<PlaybackRate>>,
        tween: Tween,
    ) {
        let mut state = self.0.lock();
        match state.deref_mut() {
            SoundState::Handle(handle) => handle.set_playback_rate(playback_rate, tween),
            #[cfg(not(target_arch = "wasm32"))]
            SoundState::StreamingHandle(handle) => handle.set_playback_rate(playback_rate, tween),
            SoundState::Deferred(commands) => {
                commands.push_back(DeferredSoundCommand::SetPlaybackRate {
                    playback_rate: playback_rate.into(),
                    tween,
                })
            }
        }
    }

    /// set the distribution of the sound between the left and right stereo outputs
    pub fn set_panning(&mut self, panning: impl Into<Value<Panning>>, tween: Tween) {
        let mut state = self.0.lock();
        match state.deref_mut() {
            SoundState::Handle(handle) => handle.set_panning(panning, tween),
            #[cfg(not(target_arch = "wasm32"))]
            SoundState::StreamingHandle(handle) => handle.set_panning(panning, tween),
            SoundState::Deferred(commands) => {
                commands.push_back(DeferredSoundCommand::SetPanning {
                    panning: panning.into(),
                    tween,
                })
            }
        }
    }

    /// set where the sound starts and stops
    pub fn set_loop_region(&mut self, region: impl IntoOptionalRegion) {
        let mut state = self.0.lock();
        match state.deref_mut() {
            SoundState::Handle(handle) => handle.set_loop_region(region),
            #[cfg(not(target_arch = "wasm32"))]
            SoundState::StreamingHandle(handle) => handle.set_loop_region(region),
            SoundState::Deferred(commands) => commands.push_back(
                DeferredSoundCommand::SetLoopReigon(region.into_optional_region()),
            ),
        }
    }

    /// stops the sound from playing while keeping the current position
    ///
    /// tweening fades the sound out
    pub fn pause(&mut self, tween: Tween) {
        let mut state = self.0.lock();
        match state.deref_mut() {
            SoundState::Handle(handle) => handle.pause(tween),
            #[cfg(not(target_arch = "wasm32"))]
            SoundState::StreamingHandle(handle) => handle.pause(tween),
            SoundState::Deferred(commands) => {
                commands.push_back(DeferredSoundCommand::Pause(tween))
            }
        }
    }

    /// resumes the sound where it was
    ///
    /// tweening fades the sound in
    pub fn resume(&mut self, tween: Tween) {
        let mut state = self.0.lock();
        match state.deref_mut() {
            SoundState::Handle(handle) => handle.resume(tween),
            #[cfg(not(target_arch = "wasm32"))]
            SoundState::StreamingHandle(handle) => handle.resume(tween),
            SoundState::Deferred(commands) => {
                commands.push_back(DeferredSoundCommand::Resume(tween))
            }
        }
    }

    /// resumes the sound at a specified position
    pub fn resume_at(&mut self, start_time: StartTime, tween: Tween) {
        let mut state = self.0.lock();
        match state.deref_mut() {
            SoundState::Handle(handle) => handle.resume_at(start_time, tween),
            #[cfg(not(target_arch = "wasm32"))]
            SoundState::StreamingHandle(handle) => handle.resume_at(start_time, tween),
            SoundState::Deferred(commands) => {
                commands.push_back(DeferredSoundCommand::ResumeAt { start_time, tween })
            }
        }
    }

    /// stops the sound and cannot be restarted
    pub fn stop(&mut self, tween: Tween) {
        let mut state = self.0.lock();
        match state.deref_mut() {
            SoundState::Handle(handle) => handle.stop(tween),
            #[cfg(not(target_arch = "wasm32"))]
            SoundState::StreamingHandle(handle) => handle.stop(tween),
            SoundState::Deferred(commands) => commands.push_back(DeferredSoundCommand::Stop(tween)),
        }
    }
}
