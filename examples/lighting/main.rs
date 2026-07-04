use maple::prelude::*;
use maple_3d::{
    assets::{
        materials::pbr_material::PbrMaterial,
        primitives::{cuboid::Cuboid, plane::Plane, sphere::Sphere},
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
                "Camera",
                Camera3D::builder()
                    .position((-10.0, 10.0, 10.0))
                    .far_plane(100.0)
                    .look_at(Vec3::ZERO)
                    .build(),
            )
            .on::<Ready>(|ctx| {
                ctx.game.get_resource_mut::<Input>().set_cursor_locked(true);
            })
            .on::<Update>(Camera3D::free_fly(1.0, 1.0));

        scene.spawn(
            "floor",
            MeshInstance3D::builder()
                .mesh(assets.add(Plane {
                    size: Vec2 { x: 100.0, y: 100.0 },
                    ..Default::default()
                }))
                .material(assets.add(PbrMaterial::default().with_base_color_factor(Color::GREY)))
                .position((0.0, -5.0, 0.0))
                .build(),
        );

        scene
            .spawn("model", Empty::builder().scale_factor(2.0).build())
            .merge_asset(assets.load::<GltfScene>("res/DamagedHelmet.glb"));

        scene.spawn(
            "world",
            Environment::new(assets.load("res/kloofendal_48d_partly_cloudy_puresky_4k.hdr"))
                .with_ibl_strength(0.1),
        );

        scene
            .spawn("pivot", Empty::builder().position((0.0, 1.0, 0.0)).build())
            .on::<FixedUpdate>(|ctx| {
                ctx.node_mut().transform.rotate_euler_xyz((0.0, 1.0, 0.0));
            })
            .with(|pivot| {
                pivot
                    .spawn_child(
                        "direct",
                        PointLight::builder()
                            .position((-2.5, 0.0, -2.5))
                            .color(Color::WHITE)
                            .intensity(10.0)
                            .build(),
                    )
                    .spawn_child(
                        "light_mesh",
                        MeshInstance3D::builder()
                            .mesh(assets.add(Sphere::default().radius(0.1)))
                            .material(
                                assets.add(
                                    PbrMaterial::default()
                                        .with_base_color_factor(Color::WHITE)
                                        .with_emissive_factor(Color::WHITE.with_intensity(10.0)),
                                ),
                            )
                            .build(),
                    );
            })
            .with(|pivot| {
                pivot
                    .spawn_child(
                        "direct",
                        PointLight::builder()
                            .position((2.5, 0.0, -2.5))
                            .color(Color::BLUE)
                            .intensity(10.0)
                            .build(),
                    )
                    .spawn_child(
                        "light_mesh",
                        MeshInstance3D::builder()
                            .mesh(assets.add(Sphere::default().radius(0.1)))
                            .material(
                                assets.add(
                                    PbrMaterial::default()
                                        .with_base_color_factor(Color::BLUE)
                                        .with_emissive_factor(Color::BLUE.with_intensity(10.0)),
                                ),
                            )
                            .build(),
                    );
            })
            .with(|pivot| {
                pivot
                    .spawn_child(
                        "direct",
                        PointLight::builder()
                            .position((-2.5, 0.0, 2.5))
                            .color(Color::RED)
                            .intensity(10.0)
                            .build(),
                    )
                    .spawn_child(
                        "light_mesh",
                        MeshInstance3D::builder()
                            .mesh(assets.add(Sphere::default().radius(0.1)))
                            .material(
                                assets.add(
                                    PbrMaterial::default()
                                        .with_base_color_factor(Color::RED)
                                        .with_emissive_factor(Color::RED.with_intensity(10.0)),
                                ),
                            )
                            .build(),
                    );
            })
            .with(|pivot| {
                pivot
                    .spawn_child(
                        "direct",
                        PointLight::builder()
                            .position((2.5, 0.0, 2.5))
                            .color(Color::GREEN)
                            .intensity(10.0)
                            .build(),
                    )
                    .spawn_child(
                        "light_mesh",
                        MeshInstance3D::builder()
                            .mesh(assets.add(Sphere::default().radius(0.1)))
                            .material(
                                assets.add(
                                    PbrMaterial::default()
                                        .with_base_color_factor(Color::GREEN)
                                        .with_emissive_factor(Color::GREEN.with_intensity(10.0)),
                                ),
                            )
                            .build(),
                    );
            });

        scene
    }
}
