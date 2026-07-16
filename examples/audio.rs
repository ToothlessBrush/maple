use std::{ops::AddAssign, time::Duration};

pub use maple::prelude::*;

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
            let handle = ctx.node_mut().play(
                ctx.assets().load("res/Week 13 - Primordial Soup BASE.ogg"),
                SoundSettings {
                    fade_in_tween: Some(Tween {
                        duration: Duration::from_secs(5),
                        ..Default::default()
                    }),

                    ..Default::default()
                },
            );

            ctx.node_handle().spawn_child(Container::new(handle));
            ctx.node_handle().spawn_child(Container::new(0f32));
        })
        .on::<Update>(|ctx| {
            let Some(mut node) = ctx.first_child::<Container<SoundHandle>>().write() else {
                return;
            };

            let input = ctx.get_resource::<Input>();
            let Some(mut volume) = ctx.first_child::<Container<f32>>().write() else {
                return;
            };
            if input.keys.contains(&KeyCode::ArrowUp) {
                volume.add_assign(ctx.event.dt * 10.0);
            }
            if input.keys.contains(&KeyCode::ArrowDown) {
                volume.add_assign(-ctx.event.dt * 10.0);
            }
            **volume = volume.clamp(-60.0, 20.0);
            node.set_volume(
                Decibels(**volume),
                Tween {
                    duration: Duration::from_secs_f32(ctx.event.dt),
                    ..Default::default()
                },
            );
        })
        .spawn_child(
            MeshInstance3D::builder()
                .mesh(assets.add(Cuboid::default()))
                .material(assets.add(Color::RED)),
        );

    scene
}
