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
        let mut scene = Scene::default();

        scene.add(
            "skybox",
            Environment::new(Path::new("res/kloofendal_48d_partly_cloudy_puresky_4k.hdr")),
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
                .orientation_vector(
                    Vec3::ZERO
                        - Vec3 {
                            x: -10.0,
                            y: 1.0,
                            z: 0.0,
                        },
                )
                .on(Ready, |ctx: &mut GameContext| {
                    ctx.get_resource_mut::<InputManager>()
                        .unwrap()
                        .set_cursor_locked(true);
                })
                .on(Update, Camera3D::free_fly(1.0, 0.5))
                .build(),
        );

        let model = Scene::load_gltf("/home/toothless/dev/maple/res/models/csr3_pagani_utopia.glb");
        let material = Scene::load_gltf_materials(
            "/home/toothless/dev/maple/res/models/red_laterite_soil_stones_1k.gltf/red_laterite_soil_stones_1k.gltf",
        );

        scene.add(
            "model",
            Empty::builder()
                .child_scene(model)
                .scale_factor(1.0)
                .build(),
        );

        scene.add(
            "ground",
            Mesh3D::plane()
                .position(Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                })
                .material(
                    material
                        .first()
                        .unwrap()
                        .clone()
                        .with_texture_scale((1000.0, 1000.0)),
                )
                .scale_factor(100.0)
                .build(),
        );

        scene.add(
            "direct",
            DirectionalLight::builder()
                .direction(Vec3 {
                    x: -1.0,
                    y: -1.0,
                    z: -1.0,
                })
                .intensity(1.0)
                .bias(0.0001)
                .build(),
        );

        scene
    }
}
