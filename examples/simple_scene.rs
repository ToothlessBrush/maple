pub use maple::prelude::*;
use maple_egui::{
    egui,
    plugin::{EguiPlugin, EguiUpdate},
};

fn main() {
    App::default()
        .add_plugin(Core3D)
        .add_plugin(EguiPlugin)
        .load_scene(scene)
        .run()
}

fn scene(assets: &AssetLibrary) -> Scene {
    let scene = Scene::default();

    scene
        .spawn(Empty::default())
        .on::<FixedUpdate>(|ctx| {
            ctx.node_mut().transform.rotate((0.1, 1.0, 0.1), 0.1);
        })
        .spawn_child(DirectionalLight::builder().direction((1.0, -1.0, -1.0)))
        .on::<EguiUpdate>(|ctx| {
            let mut node = ctx.node_mut();

            egui::Window::new("bias").show(&ctx, |ui| {
                ui.add(egui::Slider::new(&mut node.bias, -0.1..=10.0).text("bias"));
                ui.add(egui::Slider::new(&mut node.size, 0.0..=1.0).text("size"));
                ui.add(egui::Slider::new(&mut node.normal_bias, 0.0..=10.0).text("normal_bias"));
            });
        });

    scene
        .spawn(
            Camera3D::builder()
                .position((50.0, 50.0, 50.0))
                .far_plane(100.0)
                .looking_at(Vec3::ZERO),
        )
        .on::<EguiUpdate>(|ctx| {
            let mut node = ctx.node_mut();

            egui::Window::new("camera").show(&ctx, |ui| {
                ui.add(egui::Slider::new(&mut node.fov, -45.0..=120.0).text("bias"));
                ui.add(egui::Slider::new(&mut node.far, 100.0..=1000.0).text("far"));
            });
        });

    scene.spawn(
        MeshInstance3D::builder()
            .mesh(assets.add(Plane::default().size((100.0, 100.0))))
            .material(assets.add(Color::WHITE)),
    );

    scene.spawn(
        MeshInstance3D::builder()
            .mesh(assets.add(Cuboid {
                hx: 1.0,
                hz: 1.0,
                hy: 100.0,
            }))
            .material(assets.add(Color::BLUE))
            .position((0.0, 5.0, 0.0)),
    );

    scene
}
