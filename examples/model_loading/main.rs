use std::{f32::consts::PI, path::Path};

use maple::prelude::*;
use maple_3d::nodes::environment::Environment;

fn main() {
    App::new(Config::default())
        .add_plugin(Core3D)
        .load_scene(MainScene)
        .run();
}

pub struct MainScene;

impl SceneBuilder for MainScene {
    fn build(&mut self) -> Scene {
        let scene = Scene::default();

        scene.add(
            "skybox",
            Environment::new(Path::new("res/kloofendal_48d_partly_cloudy_puresky_4k.hdr"))
                .with_ibl_strength(1.0),
        );

        scene.add(
            "Camera",
            Camera3D::builder()
                .fov(PI / 2.0)
                .position(Vec3 {
                    x: -10.0,
                    y: 1.0,
                    z: 0.0,
                })
                .far_plane(100.0)
                .near_plane(0.01)
                .orientation_vector(
                    Vec3::ZERO
                        - Vec3 {
                            x: -10.0,
                            y: 1.0,
                            z: 0.0,
                        },
                )
                .on::<Ready>(|ctx| {
                    ctx.game.get_resource_mut::<Input>().set_cursor_locked(true);
                })
                .on::<Update>(Camera3D::free_fly(1.0, 0.5))
                .on::<Update>(|ctx| {
                    let fps = ctx.game.get_resource::<Frame>().fps;
                    println!("fps: {}", fps);
                })
                .build(),
        );

        let model = Scene::load_gltf("res/models/csr3_pagani_utopia.glb");

        scene
            .add("model", Empty::builder().scale_factor(1.0).build())
            .merge_scene(model);

        // scene.add(
        //     "ground",
        //     Mesh3D::plane()
        //         .position(Vec3 {
        //             x: 0.0,
        //             y: 0.0,
        //             z: 0.0,
        //         })
        //         .material(
        //             material
        //                 .first()
        //                 .unwrap()
        //                 .clone()
        //                 .with_texture_scale((100.0, 100.0)),
        //         )
        //         .scale_factor(100.0)
        //         .build(),
        // );

        scene.add(
            "direct",
            DirectionalLight::builder()
                .direction(Vec3 {
                    x: -1.0,
                    y: -1.0,
                    z: -1.0,
                })
                .intensity(0.5)
                .bias(0.0001)
                .build(),
        );

        scene
    }
}
