use std::f32::consts::PI;

use maple::prelude::*;
use maple_3d::{
    assets::{
        materials::pbr_material::PbrMaterial,
        primitives::{cuboid::Cuboid, torus::Torus},
    },
    nodes::mesh_instance::MeshInstance3D,
};

fn main() {
    App::new(Config::default())
        .add_plugin(Core3D)
        .add_plugin(Physics3D)
        .load_scene(MainScene)
        .run();
}

pub struct MainScene;

impl SceneBuilder for MainScene {
    fn build(&mut self, assets: &AssetLibrary) -> Scene {
        let scene = Scene::default();

        // scene.spawn(
        //     "skybox",
        //     Environment::new(Path::new("res/kloofendal_48d_partly_cloudy_puresky_4k.hdr"))
        //         .with_ibl_strength(1.0),
        // );

        scene
            .spawn(
                Camera3D::builder()
                    .position((-10.0, 1.0, 0.0))
                    .far_plane(100.0)
                    .look_at(Vec3::ZERO)
                    .fov(PI / 2.0)
                    .build(),
            )
            .on::<Ready>(|ctx| {
                ctx.game.get_resource_mut::<Input>().set_cursor_locked(true);
            })
            .on::<Update>(Camera3D::free_fly(1.0, 1.0));

        scene.spawn(
            MeshInstance3D::builder()
                .mesh(assets.add(Cuboid::default()))
                .material(
                    assets.add(
                        PbrMaterial::default()
                            .with_base_color_factor(Color::RED)
                            .with_emissive_factor(Color::RED.with_intensity(10.0)),
                    ),
                )
                .position((0.0, 0.0, -5.0))
                .build(),
        );

        scene.spawn(
            MeshInstance3D::builder()
                .mesh(assets.add(Torus {
                    sides: 36,
                    rings: 36,
                    ring_radius: 0.5,
                    outer_radius: 1.0,
                }))
                .material(
                    assets.add(
                        PbrMaterial::default()
                            .with_base_color_factor(Color::GREEN)
                            .with_emissive_factor(Color::GREEN.with_intensity(10.0)),
                    ),
                )
                .position((0.0, 0.0, 0.0))
                .build(),
        );

        scene.spawn(
            MeshInstance3D::builder()
                .mesh(assets.add(Cuboid::default()))
                .material(
                    assets.add(
                        PbrMaterial::default()
                            .with_base_color_factor(Color::BLUE)
                            .with_emissive_factor(Color::BLUE.with_intensity(10.0)),
                    ),
                )
                .position((0.0, 0.0, 5.0))
                .build(),
        );

        scene.spawn(
            MeshInstance3D::builder()
                .mesh(assets.add(Cuboid::default()))
                .material(
                    assets.add(
                        PbrMaterial::default()
                            .with_base_color_factor(Color::WHITE)
                            .with_emissive_factor(Color::WHITE.with_intensity(10.0)),
                    ),
                )
                .position((0.0, 0.0, 10.0))
                .build(),
        );

        scene.spawn(
            MeshInstance3D::builder()
                .mesh(assets.add(Cuboid::default()))
                .material(assets.add(PbrMaterial::default().with_base_color_factor(Color::RED)))
                .position((0.0, -2.5, -5.0))
                .build(),
        );

        scene.spawn(
            MeshInstance3D::builder()
                .mesh(assets.add(Cuboid::default()))
                .material(assets.add(PbrMaterial::default().with_base_color_factor(Color::GREEN)))
                .position((0.0, -2.5, 0.0))
                .build(),
        );

        scene.spawn(
            MeshInstance3D::builder()
                .mesh(assets.add(Cuboid::default()))
                .material(assets.add(PbrMaterial::default().with_base_color_factor(Color::BLUE)))
                .position((0.0, -2.5, 5.0))
                .build(),
        );

        scene.spawn(
            MeshInstance3D::builder()
                .mesh(assets.add(Cuboid::default()))
                .material(assets.add(PbrMaterial::default().with_base_color_factor(Color::WHITE)))
                .position((0.0, -2.5, 10.0))
                .build(),
        );

        scene.spawn(DirectionalLight::builder().build());

        scene
    }
}
