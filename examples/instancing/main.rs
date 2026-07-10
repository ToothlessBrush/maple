use maple::prelude::*;
use maple_3d::{
    assets::{
        materials::pbr_material::PbrMaterial,
        mesh::Mesh3D,
        primitives::{cuboid::Cuboid, plane::Plane, sphere::Sphere, torus::Torus},
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
    fn build(&mut self, assets: &AssetLibrary) -> Scene {
        let scene = Scene::default();

        scene
            .spawn(
                Camera3D::builder()
                    .position((-35.0, 50.0, -35.0))
                    .look_at((15.0, 15.0, 15.0))
                    .build(),
            )
            .on::<Update>(Camera3D::free_fly(5.0, 0.5));

        scene.spawn(DirectionalLight::builder().intensity(10.0).build());

        let mesh = assets.add::<Mesh3D>(Torus::default());
        let material = assets.add::<Material>(
            PbrMaterial::default()
                // .with_shadows(false)
                .with_base_color_factor(Color::GREEN),
        );

        for x in 0..10 {
            for y in 0..10 {
                for z in 0..10 {
                    scene
                        .spawn(
                            MeshInstance3D::builder()
                                .mesh(mesh.clone())
                                .material(material.clone())
                                .position((x as f32 * 3.0, y as f32 * 3.0, z as f32 * 3.0))
                                .rotation_euler_xyz((
                                    x as f32 * 10.0,
                                    y as f32 * 15.0,
                                    z as f32 * 20.0,
                                ))
                                .build(),
                        )
                        .on::<FixedUpdate>(|ctx| {
                            ctx.node_mut()
                                .transform
                                .rotate_euler_xyz((0.25, 0.25, 0.25));
                        });
                }
            }
        }

        scene
    }
}
