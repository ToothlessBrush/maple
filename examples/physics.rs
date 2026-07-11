use maple::prelude::*;
use maple_3d::{
    assets::{materials::pbr_material::PbrMaterial, primitives::cuboid::Cuboid},
    nodes::mesh_instance::MeshInstance3D,
};

fn main() {
    App::new(Config {
        ..Default::default()
    })
    .add_plugin(Core3D)
    .add_plugin(Physics3D)
    .load_scene(PhysicsScene)
    .run();
}

pub struct PhysicsScene;

impl SceneBuilder for PhysicsScene {
    fn build(self, assets: &AssetLibrary) -> Scene {
        let scene = Scene::default();

        scene.spawn(Environment::new(assets.load("res/cayley_interior_4k.hdr")));

        // Camera
        let camera = scene.spawn(
            Camera3D::builder()
                .position(Vec3::new(-40.0, 40.0, -40.0))
                .looking_at((0.0, -10.0, 0.0))
                .far_plane(500.0)
                .build(),
        );
        camera
            .on::<Ready>(|ctx| {
                ctx.get_resource_mut::<Input>().set_cursor_locked(true);
            })
            .on::<Update>(Camera3D::free_fly(5.0, 0.5))
            .on::<Update>(|ctx| {
                let input = ctx.get_resource::<Input>();
                if input.mouse_button_just_pressed.contains(&MouseButton::Left) {
                    let transform = ctx.node.read().transform;
                    let position = transform.position;
                    let forward = transform.get_forward_vector();
                    let speed = 100.0;

                    let projectile = ctx.scene().spawn(
                        RigidBody3DBuilder::dynamic()
                            .position(position)
                            .linear_velocity(forward * speed)
                            .build(),
                    );
                    projectile.spawn_child(
                        MeshInstance3D::builder()
                            .mesh(ctx.assets().add(Cuboid::default()))
                            .material(
                                ctx.assets().add(
                                    PbrMaterial::default().with_base_color_factor(Color::BLUE),
                                ),
                            )
                            .scale_factor(0.1)
                            .build(),
                    );
                    projectile.spawn_child(Collider3DBuilder::ball(0.5).mass(10.0).build());
                }
            });

        // Light
        scene.spawn(
            DirectionalLight::builder()
                .direction(Vec3::new(0.0, -1.0, 0.0))
                .intensity(1.0)
                .build(),
        );

        // Ground - static rigid body with box collider
        let ground = scene.spawn(
            RigidBody3DBuilder::fixed()
                .position(Vec3::new(0.0, -1.0, 0.0))
                .build(),
        );
        ground.spawn_child(
            MeshInstance3D::builder()
                .mesh(assets.add(Cuboid::default()))
                .material(
                    assets.add(
                        PbrMaterial::default()
                            .with_base_color_factor(Color::WHITE)
                            .with_metallic_factor(0.5)
                            .with_roughness_factor(0.5),
                    ),
                )
                .scale(Vec3 {
                    x: 10000.0,
                    y: 1.0,
                    z: 10000.0,
                })
                .build(),
        );
        ground.spawn_child(
            Collider3DBuilder::cuboid(5000.0, 0.5, 5000.0)
                .friction(1000.0)
                .build(),
        );

        let cube_mesh = assets.add(Cuboid::default());
        let material = assets.add(
            PbrMaterial::default()
                .with_base_color_factor(Color::GREY)
                .with_roughness_factor(0.2)
                .with_metallic_factor(0.2),
        );
        for x in 0..10 {
            for y in 0..10 {
                for z in 0..10 {
                    let body = scene.spawn(
                        RigidBody3DBuilder::dynamic()
                            .position(Vec3::new(x as f32, y as f32, z as f32))
                            .build(),
                    );
                    body.spawn_child(
                        MeshInstance3D::builder()
                            .mesh(cube_mesh.clone())
                            .material(material.clone())
                            .build(),
                    );
                    body.spawn_child(Collider3DBuilder::cuboid(0.5, 0.5, 0.5).build());
                }
            }
        }

        scene
    }
}
