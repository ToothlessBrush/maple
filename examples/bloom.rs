use std::f32::consts::PI;

use maple::prelude::*;

fn main() {
    App::new(Config::default())
        .add_plugin(Core3D)
        .add_plugin(Physics3D)
        .load_scene(MainScene)
        .run();
}

pub struct MainScene;

impl SceneBuilder for MainScene {
    fn build(self, assets: &AssetLibrary) -> Scene {
        let scene = Scene::default();

        scene.spawn(
            Camera3D::builder()
                .position((-10.0, 2.0, 2.5))
                .far_plane(100.0)
                .looking_at((0.1, 0.1, 2.5))
                .fov(75.0),
        );

        scene.spawn(
            MeshInstance3D::builder()
                .mesh(assets.add(Cuboid::default()))
                .material(assets.add(PbrMaterial {
                    base_color_factor: Color::RED,
                    emissive_factor: Color::RED.with_intensity(10.0),
                    ..Default::default()
                }))
                .position((0.0, 0.0, -5.0)),
        );

        scene.spawn(
            MeshInstance3D::builder()
                .mesh(assets.add(Torus {
                    sides: 36,
                    rings: 36,
                    ring_radius: 0.5,
                    outer_radius: 1.0,
                }))
                .material(assets.add(PbrMaterial {
                    base_color_factor: Color::GREEN,
                    emissive_factor: Color::GREEN.with_intensity(10.0),
                    ..Default::default()
                }))
                .position((0.0, 0.0, 0.0)),
        );

        scene.spawn(
            MeshInstance3D::builder()
                .mesh(assets.add(Sphere::default()))
                .material(assets.add(PbrMaterial {
                    base_color_factor: Color::BLUE,
                    emissive_factor: Color::BLUE.with_intensity(10.0),
                    ..Default::default()
                }))
                .position((0.0, 0.0, 5.0)),
        );

        scene.spawn(
            MeshInstance3D::builder()
                .mesh(assets.add(Cuboid::default()))
                .material(assets.add(PbrMaterial {
                    base_color_factor: Color::WHITE,
                    emissive_factor: Color::WHITE.with_intensity(10.0),
                    ..Default::default()
                }))
                .position((0.0, 0.0, 10.0)),
        );

        scene.spawn(
            MeshInstance3D::builder()
                .mesh(assets.add(Cuboid::default()))
                .material(assets.add(PbrMaterial {
                    base_color_factor: Color::RED,
                    ..Default::default()
                }))
                .position((0.0, -2.5, -5.0)),
        );

        scene.spawn(
            MeshInstance3D::builder()
                .mesh(assets.add(Torus::default()))
                .material(assets.add(PbrMaterial {
                    base_color_factor: Color::GREEN,
                    ..Default::default()
                }))
                .position((0.0, -2.5, 0.0)),
        );

        scene.spawn(
            MeshInstance3D::builder()
                .mesh(assets.add(Sphere::default()))
                .material(assets.add(PbrMaterial {
                    base_color_factor: Color::BLUE,
                    ..Default::default()
                }))
                .position((0.0, -2.5, 5.0)),
        );

        scene.spawn(
            MeshInstance3D::builder()
                .mesh(assets.add(Cuboid::default()))
                .material(assets.add(PbrMaterial {
                    base_color_factor: Color::WHITE,
                    ..Default::default()
                }))
                .position((0.0, -2.5, 10.0)),
        );

        scene.spawn(DirectionalLight::builder().direction((-1.0, -1.0, -1.0)));

        scene
    }
}
