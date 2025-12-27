use std::path::{Path, PathBuf};

use maple::prelude::{node::Casting, *};
use maple_3d::{
    components::material::MaterialProperties,
    gltf::GLTFLoader,
    nodes::{camera::Camera3D, directional_light::DirectionalLight, mesh::Mesh3D},
};
use maple_engine::components::event_reciever::{Ready, Update};
use maple_physics::nodes::{Collider3DBuilder, RigidBody3D, RigidBody3DBuilder};
use maple_renderer::core::texture::{self, LazyTexture, Texture};

pub struct MainScene;

impl SceneBuilder for MainScene {
    fn build(&mut self) -> Scene {
        let mut scene = Scene::default();

        scene.add(
            "Camera",
            Camera3D::builder()
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
                .on(Update, Camera3D::free_fly(1.0, 1.0))
                .build(),
        );

        scene.add(
            "ball",
            RigidBody3DBuilder::dynamic()
                .add_child(
                    "collider",
                    Collider3DBuilder::ball(1.0).restitution(1.5).build(),
                )
                .add_child(
                    "mesh",
                    Mesh3D::smooth_sphere()
                        .material(MaterialProperties::default().with_base_color_factor(Color::RED))
                        .build(),
                )
                .position(Vec3 {
                    x: 0.0,
                    y: 30.0,
                    z: 0.0,
                })
                .build(),
        );

        scene.add(
            "floor",
            RigidBody3DBuilder::fixed()
                .add_child(
                    "Collider",
                    Collider3DBuilder::cuboid(10.0, 1.0, 10.0).build(),
                )
                .position(Vec3 {
                    x: 0.0,
                    y: -2.0,
                    z: 0.0,
                })
                .build(),
        );
        scene.add(
            "ground",
            Mesh3D::plane()
                .position(Vec3 {
                    x: 0.0,
                    y: -1.0,
                    z: 0.0,
                })
                .scale_factor(9.0)
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
