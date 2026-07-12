use maple::prelude::*;
use maple_3d::{
    assets::{
        materials::pbr_material::PbrMaterial,
        primitives::{plane::Plane, torus::Torus},
    },
    gltf::GltfScene,
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
    fn build(self, assets: &AssetLibrary) -> Scene {
        let scene = Scene::default();

        scene.spawn(
            Camera3D::builder()
                .position((-10.0, 10.0, -10.0))
                .looking_at(Vec3::ZERO),
        );

        scene
            .spawn(Empty::default())
            .on::<FixedUpdate>(|ctx| {
                ctx.node_mut().transform.rotate_euler_xyz((0.0, 0.0, 1.0));
            })
            .spawn_child(
                DirectionalLight::builder()
                    .direction((-1.0, -1.0, 1.0))
                    .intensity(10.0),
            );

        let material = assets.map(
            assets.load::<GltfScene>("res/models/dark_rock_4k.gltf/dark_rock_4k.gltf"),
            |gltf| gltf.get_material(0),
        );

        assets.modify(&material, |mat| {
            mat.get_instance_mut::<PbrMaterial>().unwrap().texture_scale =
                Vec2 { x: 10.0, y: 10.0 };
            mat.get_instance_mut::<PbrMaterial>().unwrap().cull_mode =
                maple_renderer::core::CullMode::None;
        });

        scene.spawn(
            MeshInstance3D::builder()
                .mesh(assets.add(Plane::default()))
                .material(material)
                .position((0.0, -2.5, 0.0))
                .scale_factor(100.0),
        );

        scene
            .spawn(
                MeshInstance3D::builder()
                    .mesh(assets.add(Torus::default().ring_radius(0.5).outer_radius(2.0)))
                    .scale_factor(2.5)
                    .material(assets.add(assets.load("res/2k_earth_daymap.jpg"))),
            )
            .on::<FixedUpdate>(|ctx| {
                ctx.node_mut().transform.rotate_euler_xyz((0.0, 0.1, 0.0));
            });

        scene
    }
}
