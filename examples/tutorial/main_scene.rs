use maple::prelude::*;
use maple_3d::nodes::{
    camera::Camera3D, directional_light::DirectionalLight, mesh::Mesh3D, point_light::PointLight,
};
use maple_engine::components::event_reciever::{Ready, Update};

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

        scene.add("block", Mesh3D::cube().build());
        scene.add(
            "ground",
            Mesh3D::cube()
                .position(Vec3 {
                    x: 0.0,
                    y: -5.0,
                    z: 0.0,
                })
                .scale_factor(9.0)
                .build(),
        );

        scene.add(
            "light",
            DirectionalLight::builder()
                .direction(Vec3 {
                    x: -1.0,
                    y: -1.0,
                    z: 0.01,
                })
                .intensity(1.0)
                .build(),
        );

        // scene.add(
        //     "point",
        //     PointLight::builder()
        //         .position(Vec3 {
        //             x: 0.0,
        //             y: 2.0,
        //             z: 0.0,
        //         })
        //         .intensity(100.0)
        //         .build(),
        // );

        scene
    }
}
