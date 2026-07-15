use glam::{Quat, Vec3};
use kira::{AudioManagerSettings, DefaultBackend, Tween, track::SpatialTrackBuilder};
use maple_app::Plugin;
use maple_engine::prelude::Frame;

use crate::{
    asset::{AudioData, AudioLoader},
    nodes::{audio_listener::AudioListener, audio_source::AudioSource},
    resource::AudioManager,
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

        for (audio, settings) in std::mem::take(&mut manager.queue) {
            let Some(data) = app.context().assets.get(&audio) else {
                manager.queue.push_back((audio, settings));
                continue;
            };
            match &data.data {
                AudioData::Static(sound_data) => {
                    let _handle = manager
                        .manager
                        .play(sound_data.clone().with_settings(settings.into()));
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
            let handle = source.handle.get_or_insert_with(|| {
                manager
                    .manager
                    .add_spatial_sub_track(id, Vec3::default(), SpatialTrackBuilder::default())
                    .unwrap()
            });

            handle.set_position(source.transform.world_space().position, Tween::default());

            for (audio, settings) in std::mem::take(&mut source.queue) {
                let Some(data) = app.context().assets.get(&audio) else {
                    source.queue.push_back((audio, settings)); // not loaded
                    continue;
                };
                match &data.data {
                    AudioData::Static(sound_data) => {
                        let _handle =
                            handle.play(sound_data.clone().with_settings(settings.into()));
                    }
                    AudioData::Streaming { .. } => continue, // TODO
                }
            }
        })
    }
}
