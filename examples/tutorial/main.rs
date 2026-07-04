use maple::prelude::*;
use maple_3d::{
    assets::{
        materials::pbr_material::PbrMaterial,
        primitives::{cuboid::Cuboid, plane::Plane, sphere::Sphere},
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

        scene
            .spawn(
                "Camera",
                Camera3D::builder()
                    .position((-20.0, 20.0, 20.0))
                    .far_plane(100.0)
                    .look_at(Vec3::ZERO)
                    .build(),
            )
            .on::<Ready>(|ctx| {
                ctx.game.get_resource_mut::<Input>().set_cursor_locked(true);
            })
            .on::<Update>(Camera3D::free_fly(1.0, 1.0));

        scene.spawn(
            "Sphere",
            MeshInstance3D::builder()
                .mesh(assets.add(Sphere::default().radius(2.5)))
                .material(assets.add(PbrMaterial::default().with_base_color_factor(Color::RED)))
                .build(),
        );

        scene.spawn(
            "floor",
            MeshInstance3D::builder()
                .mesh(assets.add(Plane {
                    size: Vec2 { x: 100.0, y: 100.0 },
                    ..Default::default()
                }))
                .material(assets.add(PbrMaterial::default()))
                .position((0.0, -5.0, 0.0))
                .build(),
        );

        scene.spawn("sun", DirectionalLight::builder().build());

        scene
            .spawn("pivot", Empty::default())
            .on::<FixedUpdate>(|ctx| {
                ctx.node_mut().transform.rotate_euler_xyz((0.0, 1.0, 0.0));
            })
            .spawn_child(
                "direct",
                PointLight::builder()
                    .position((2.5, 2.5, 2.5))
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

        scene
    }
}
