use std::f32::consts::PI;

use maple::prelude::*;
use maple_egui::{
    egui,
    plugin::{EguiPlugin, EguiUpdate},
};

fn main() {
    App::new(Config::default())
        .add_plugin(Core3D)
        .add_plugin(EguiPlugin)
        .load_scene(MainScene)
        .run();
}

pub struct MainScene;

impl SceneBuilder for MainScene {
    fn build(self, assets: &AssetLibrary) -> Scene {
        let scene = Scene::default();

        scene.spawn(
            Environment::new(assets.load("res/kloofendal_48d_partly_cloudy_puresky_4k.hdr"))
                .with_ibl_strength(0.2),
        );

        scene
            .spawn(
                Camera3D::builder()
                    .fov(90.0)
                    .exposure(0.5)
                    .position(Vec3 {
                        x: -10.0,
                        y: 1.0,
                        z: 0.0,
                    })
                    .far_plane(100.0)
                    .near_plane(0.01)
                    .looking_at(Vec3::ZERO)
                    .build(),
            )
            .on::<EguiUpdate>(|ctx| {
                egui::Window::new("fps").show(&ctx, |ui| {
                    let fps = ctx.get_resource::<Frame>().fps;
                    ui.label(format!("fps: {}", fps));
                });
            })
            .on::<Update>(Camera3D::free_fly(1.0, 0.5))
            // .on::<Update>(|ctx| {
            //     ctx.node_mut().look_at(Vec3::ZERO);
            // })
            .on::<FixedUpdate>(|ctx| {
                let input = ctx.get_resource::<Input>();

                if input.keys.contains(&KeyCode::ArrowUp) {
                    ctx.node_mut().exposure += 0.01
                }
                if input.keys.contains(&KeyCode::ArrowDown) {
                    ctx.node_mut().exposure -= 0.01
                }
            });

        scene.spawn(
            DirectionalLight::builder()
                .direction((-0.5, -1.0, -0.5))
                .intensity(10.0)
                .build(),
        );

        let sponza = assets.load::<GltfScene>("res/DamagedHelmet.glb");

        let model = scene.spawn(Empty::builder().scale_factor(1.0).build());
        model.merge_asset(sponza);

        scene
    }
}
