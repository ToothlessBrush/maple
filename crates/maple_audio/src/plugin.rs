use std::ops::DerefMut;

use glam::{Quat, Vec3};
use kira::{AudioManagerSettings, DefaultBackend, Tween, track::SpatialTrackBuilder};
use maple_app::Plugin;
use maple_engine::prelude::Frame;

use crate::{
    asset::{AudioData, AudioLoader},
    nodes::{
        audio_listener::AudioListener,
        audio_source::{AudioSource, SourceHandle},
    },
    resource::AudioManager,
    sound::{DeferredSoundCommand, SoundState},
};

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn setup(&self, app: &mut maple_app::App<maple_app::Init>) {
        app.context_mut().insert_resource(AudioManager::new(
            kira::AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap(),
        ));

        app.context_mut().assets.register_loader(AudioLoader);
    }

    fn update(&self, app: &mut maple_app::App<maple_app::Running>) {
        let mut manager = app.context().get_resource_mut::<AudioManager>();

        for (audio, settings, handle) in std::mem::take(&mut manager.queue) {
            let Some(data) = app.context().assets.get(&audio) else {
                manager.queue.push_back((audio, settings, handle));
                continue;
            };
            match &data.data {
                AudioData::Static(sound_data) => {
                    let mut real_handle = manager
                        .manager
                        .play(sound_data.clone().with_settings(settings.into()))
                        .expect("failed to play sound");
                    let mut state = handle.0.lock();
                    if let SoundState::Deferred(commands) = state.deref_mut() {
                        DeferredSoundCommand::apply_commands(&mut real_handle, commands);
                    }
                    *state = SoundState::Handle(real_handle)
                }
                AudioData::Streaming { .. } => continue,
            }
        }

        let listeners = app.context().scene.collect::<AudioListener>();

        let Some(active_listener) = listeners
            .iter()
            .max_by_key(|listener| listener.read().priority)
        else {
            manager.listener = None;
            return;
        };

        if manager.listener.is_none() {
            let id = manager
                .manager
                .add_listener(Vec3::default(), Quat::default())
                .expect("listener to be created there should only be one listener");
            manager.listener = Some(id);
        }
        let listener = manager.listener.as_mut().unwrap();

        let tween = Tween {
            duration: app.context().get_resource::<Frame>().time_delta,
            ..Default::default()
        };

        listener.set_position(
            active_listener.read().transform.world_space().position,
            tween,
        );
        listener.set_orientation(
            active_listener.read().transform.world_space().rotation,
            tween,
        );

        let id = listener.id();

        app.context().scene.for_each::<AudioSource>(&mut |source| {
            if let SourceHandle::DeferredCommands(commands) = &mut source.handle {
                let mut handle = manager
                    .manager
                    .add_spatial_sub_track(id, Vec3::ZERO, SpatialTrackBuilder::default())
                    .expect("max spatial tracks reached");
                SourceHandle::apply_commands_spatial(&mut handle, commands);
                source.handle = SourceHandle::SpatialHandle(handle);
            }

            let SourceHandle::SpatialHandle(spatial_handle) = &mut source.handle else {
                unreachable!("just resolved above")
            };

            spatial_handle.set_position(source.transform.world_space().position, Tween::default());

            for (audio, settings, sound_handle) in std::mem::take(&mut source.queue) {
                let Some(data) = app.context().assets.get(&audio) else {
                    source.queue.push_back((audio, settings, sound_handle)); // not loaded
                    continue;
                };
                match &data.data {
                    AudioData::Static(sound_data) => {
                        let mut real_handle = spatial_handle
                            .play(sound_data.clone().with_settings(settings.into()))
                            .expect("failed to play sound");

                        let mut state = sound_handle.0.lock();
                        if let SoundState::Deferred(commands) = state.deref_mut() {
                            DeferredSoundCommand::apply_commands(&mut real_handle, commands);
                        }
                        *state = SoundState::Handle(real_handle)
                    }
                    AudioData::Streaming { .. } => continue, // TODO
                }
            }
        })
    }
}
