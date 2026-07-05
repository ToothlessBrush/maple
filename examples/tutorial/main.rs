use maple::prelude::*;
use maple_3d::{
    assets::{
        materials::pbr_material::PbrMaterial,
        primitives::{plane::Plane, torus::Torus},
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

        scene.spawn(
            Camera3D::builder()
                .position((10.0, 10.0, 10.0))
                .look_at(Vec3::ZERO)
                .build(),
        );

        scene.spawn(DirectionalLight::builder().build());

        scene.spawn(
            MeshInstance3D::builder()
                .mesh(assets.add(Plane::default()))
                .material(assets.add(PbrMaterial::default()))
                .position((0.0, -2.5, 0.0))
                .scale_factor(100.0)
                .build(),
        );

        scene.spawn(
            MeshInstance3D::builder()
                .mesh(assets.add(Torus {
                    inner_radius: 0.5,
                    outer_radius: 1.5,
                    sides: 36,
                    rings: 36,
                }))
                .material(assets.add(PbrMaterial::default()))
                .build(),
        );

        scene
    }
}
