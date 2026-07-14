use std::time::Duration;

pub use maple::prelude::*;
use maple_audio::nodes::{
    audio_listener::AudioListener,
    audio_source::{AudioSource, Region, SoundSettings, Tween},
};

fn main() {
    App::default()
        .add_plugin(Core3D)
        .add_plugin(AudioPlugin)
        .load_scene(scene)
        .run()
}

fn scene(assets: &AssetLibrary) -> Scene {
    let scene = Scene::default();

    scene.spawn(DirectionalLight::builder().direction((-1.0, -1.0, -1.0)));

    scene
        .spawn(
            Camera3D::builder()
                .position((10.0, 10.0, 10.0))
                .looking_at(Vec3::ZERO),
        )
        .on::<Update>(Camera3D::free_fly(2.0, 0.5))
        .on::<Ready>(|ctx| ctx.get_resource_mut::<Input>().set_cursor_locked(true))
        .spawn_child(AudioListener::default());

    scene
        .spawn(AudioSource::default())
        .on::<Ready>(|ctx| {
            ctx.node_mut().play(
                ctx.assets().load("res/Week 13 - Primordial Soup BASE.ogg"),
                SoundSettings {
                    loop_regions: Some(Region::default()),
                    fade_in_tween: Some(Tween {
                        duration: Duration::from_secs(5),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )
        })
        .spawn_child(
            MeshInstance3D::builder()
                .mesh(assets.add(Cuboid::default()))
                .material(assets.add(Color::RED)),
        );

    scene
}
