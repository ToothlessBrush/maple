pub use maple::prelude::*;
use maple_egui::{
    egui::{self, Panel, Ui},
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

    #[derive(Default)]
    struct EguiDemoState {
        text: String,
        multiline_text: String,
        checkbox: bool,
        radio: usize,
        slider: f32,
        drag_value: f32,
        color: [f32; 4],
        combo_selected: usize,
        progress: f32,
        selectable: usize,
    }

    scene
        .spawn(Container::new(EguiDemoState::default()))
        .on::<EguiUpdate>(|ctx| {
            let mut state = ctx.node_mut();
            let fps = ctx.get_resource::<Frame>();

            egui::Window::new("egui vanilla widgets").show(&ctx.event.ctx, |ui| {
                ui.heading("Labels & Text");
                ui.label("Hello World!");
                ui.label(format!("fps: {}", fps.fps));
                ui.label(
                    egui::RichText::new("Colored/bold text")
                        .color(egui::Color32::from_rgb(255, 128, 0))
                        .strong(),
                );
                ui.hyperlink_to("A hyperlink", "https://github.com/emilk/egui");
                ui.separator();

                ui.heading("Buttons & Toggles");
                if ui.button("Click me").clicked() {
                    println!("button clicked");
                }
                if ui.small_button("Small button").clicked() {}
                ui.checkbox(&mut state.checkbox, "Checkbox");
                ui.horizontal(|ui| {
                    ui.radio_value(&mut state.radio, 0, "Radio A");
                    ui.radio_value(&mut state.radio, 1, "Radio B");
                    ui.radio_value(&mut state.radio, 2, "Radio C");
                });
                ui.separator();

                ui.heading("Sliders & Drag Values");
                ui.add(egui::Slider::new(&mut state.slider, 0.0..=100.0).text("Slider"));
                ui.add(
                    egui::DragValue::new(&mut state.drag_value)
                        .speed(0.1)
                        .prefix("drag: "),
                );
                ui.add(egui::ProgressBar::new(state.progress).show_percentage());
                state.progress = (state.progress + 0.002).rem_euclid(1.0);
                ui.separator();

                ui.heading("Text Input");
                ui.text_edit_singleline(&mut state.text);
                ui.text_edit_multiline(&mut state.multiline_text);
                ui.separator();

                ui.heading("Color");
                ui.color_edit_button_rgba_unmultiplied(&mut state.color);
                ui.separator();

                ui.heading("Combo Box");
                egui::ComboBox::from_label("Pick one")
                    .selected_text(format!("Option {}", state.combo_selected))
                    .show_ui(ui, |ui| {
                        for i in 0..4 {
                            ui.selectable_value(
                                &mut state.combo_selected,
                                i,
                                format!("Option {i}"),
                            );
                        }
                    });
                ui.separator();

                ui.heading("Selectable Labels");
                ui.horizontal(|ui| {
                    for i in 0..3 {
                        if ui
                            .selectable_label(state.selectable == i, format!("Tab {i}"))
                            .clicked()
                        {
                            state.selectable = i;
                        }
                    }
                });
                ui.separator();

                ui.heading("Layout");
                ui.columns(2, |cols| {
                    cols[0].label("Column 1");
                    cols[1].label("Column 2");
                });

                egui::Grid::new("demo_grid").show(ui, |ui| {
                    ui.label("Row 1, Col 1");
                    ui.label("Row 1, Col 2");
                    ui.end_row();
                    ui.label("Row 2, Col 1");
                    ui.label("Row 2, Col 2");
                    ui.end_row();
                });
                ui.separator();

                ui.collapsing("Collapsing header", |ui| {
                    ui.label("Hidden content revealed!");
                });
                ui.separator();

                egui::ScrollArea::vertical()
                    .max_height(100.0)
                    .show(ui, |ui| {
                        for i in 0..20 {
                            ui.label(format!("Scrollable row {i}"));
                        }
                    });
                ui.separator();

                ui.spinner();

                ui.label("Hover me").on_hover_text("This is a tooltip");
            });
        });

    scene
        .spawn(Empty::default())
        .on::<FixedUpdate>(|ctx| {
            ctx.node_mut().transform.rotate((0.1, 1.0, 0.1), 0.1);
        })
        .spawn_child(DirectionalLight::builder().direction((1.0, -1.0, -1.0)));

    scene.spawn(
        Camera3D::builder()
            .position((2.0, 2.0, 2.0))
            .looking_at(Vec3::ZERO),
    );

    scene.spawn(
        MeshInstance3D::builder()
            .mesh(assets.add(Plane::default().size((2.0, 2.0))))
            .material(assets.add(Color::WHITE)),
    );

    scene.spawn(
        MeshInstance3D::builder()
            .mesh(assets.add(Cuboid::default().half_extent(0.1)))
            .material(assets.add(Color::BLUE))
            .position((0.0, 0.3, 0.0)),
    );

    scene
}
