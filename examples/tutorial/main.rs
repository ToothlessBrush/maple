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
    fn build(&mut self, _assets: &AssetLibrary) -> Scene {
        let scene = Scene::default();

        scene
            .spawn(
                "Camera",
                Camera3D::builder()
                    .position((-20.0, 20.0, 20.0))
                    .far_plane(100.0)
                    .orientation_vector(
                        Vec3::ZERO
                            - Vec3 {
                                x: -20.0,
                                y: 20.0,
                                z: 20.0,
                            },
                    )
                    .build(),
            )
            .on::<Ready>(|ctx| {
                ctx.game.get_resource_mut::<Input>().set_cursor_locked(true);
            })
            .on::<Update>(Camera3D::free_fly(1.0, 1.0));

        scene.spawn(
            "floor",
            Mesh3D::plane()
                .position((0.0, -2.0, 0.0))
                .scale_factor(10.0)
                .build(),
        );

        scene.spawn(
            "cube",
            Mesh3D::cube()
                .material(MaterialProperties::default().with_base_color_factor(Color::BLUE))
                .build(),
        );

        scene.spawn(
            "direct",
            DirectionalLight::builder()
                .direction((-1.0, -1.0, -1.0))
                .build(),
        );

        scene
    }
}
