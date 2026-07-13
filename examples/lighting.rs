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
    fn build(self, assets: &AssetLibrary) -> Scene {
        let scene = Scene::default();

        scene
            .spawn(
                Camera3D::builder()
                    .position((-10.0, 10.0, 10.0))
                    .far_plane(1000.0)
                    .looking_at(Vec3::ZERO)
                    .build(),
            )
            .on::<Ready>(|ctx| {
                ctx.game.get_resource_mut::<Input>().set_cursor_locked(true);
            })
            // .on::<FixedUpdate>(|ctx| println!("{}", ctx.get_resource::<Frame>().fps))
            .on::<Update>(Camera3D::free_fly(1.0, 1.0));

        scene.spawn(
            MeshInstance3D::builder()
                .mesh(assets.add(Plane {
                    size: Vec2 { x: 100.0, y: 100.0 },
                    ..Default::default()
                }))
                .material(assets.add(Color::GREY))
                .position((0.0, -5.0, 0.0))
                .build(),
        );

        scene
            .spawn(Empty::builder().scale_factor(2.0).build())
            .merge_asset(assets.load::<GltfScene>("res/DamagedHelmet.glb"));

        scene.spawn(
            Environment::new(assets.load("res/kloofendal_48d_partly_cloudy_puresky_4k.hdr"))
                .with_ibl_strength(0.1)
                .quality_medium(),
        );

        scene
            .spawn(Empty::builder().position((0.0, 1.0, 0.0)).build())
            .on::<FixedUpdate>(|ctx| {
                ctx.node_mut().transform.rotate_euler_xyz((0.0, 1.0, 0.0));
            })
            .with(|pivot| {
                let colors_pos = [
                    (Color::WHITE, (2.5, 0.0, 2.5)),
                    (Color::RED, (2.5, 0.0, -2.5)),
                    (Color::GREEN, (-2.5, 0.0, 2.5)),
                    (Color::BLUE, (-2.5, 0.0, -2.5)),
                ];

                for (color, pos) in colors_pos {
                    pivot
                        .spawn_child(
                            PointLight::builder()
                                .position(pos)
                                .color(color)
                                .intensity(10.0)
                                .build(),
                        )
                        .spawn_child(
                            MeshInstance3D::builder()
                                .mesh(assets.add(Sphere::default().radius(0.1)))
                                .material(assets.add(PbrMaterial {
                                    base_color_factor: color,
                                    cast_shadows: false,
                                    emissive_factor: color.with_intensity(20.0),
                                    ..Default::default()
                                }))
                                .build(),
                        );
                }
            });

        scene
    }
}
