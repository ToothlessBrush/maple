use maple::prelude::*;
use maple_3d::{
    assets::{
        materials::pbr_material::PbrMaterial,
        primitives::{plane::Plane, sphere::Sphere, torus::Torus},
    },
    gltf::GltfScene,
    nodes::mesh_instance::MeshInstance3D,
};
use maple_engine::asset::AssetState;

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
                .position((-10.0, 10.0, -10.0))
                .look_at(Vec3::ZERO)
                .build(),
        );

        scene.spawn(DirectionalLight::builder().intensity(10.0).build());
        scene.spawn(
            DirectionalLight::builder()
                .direction((0.0, -1.0, 0.0))
                .intensity(10.0)
                .build(),
        );

        scene
            .spawn(
                MeshInstance3D::builder()
                    .mesh(assets.add(Plane::default()))
                    .material(assets.map(
                        assets.load::<GltfScene>("res/models/dark_rock_4k.gltf/dark_rock_4k.gltf"),
                        |gltf| gltf.get_material(0),
                    ))
                    .position((0.0, -2.5, 0.0))
                    .scale_factor(100.0)
                    .build(),
            )
            .on::<Update>(|ctx| {
                if ctx.node().material.is_some() {
                    return;
                }
                let material = ctx
                    .scene()
                    .get::<Container<AssetHandle<GltfScene>>>(*ctx.node.children().first().unwrap())
                    .unwrap();

                if let AssetState::Loaded(material) = ctx.assets().get(material.read().get_item()) {
                    ctx.node_mut().material = material.get_material(0);
                }
            })
            .spawn_child(Container::new(
                assets.load::<GltfScene>("res/models/dark_rock_4k.gltf/dark_rock_4k.gltf"),
            ));

        scene
            .spawn(
                MeshInstance3D::builder()
                    .mesh(assets.add(Torus::default()))
                    .scale_factor(2.5)
                    .material(
                        assets.add(
                            PbrMaterial::default()
                                .with_base_color_texture(assets.load("res/2k_earth_daymap.jpg")),
                        ),
                    )
                    .build(),
            )
            .on::<FixedUpdate>(|ctx| {
                ctx.node_mut().transform.rotate_euler_xyz((0.0, 0.1, 0.0));
            });

        scene
    }
}
