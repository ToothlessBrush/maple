use std::ops::Deref;

use egui::{Color32, Context, FullOutput};
use maple_app::Plugin;
use maple_engine::{
    color::Color,
    prelude::{EventLabel, Frame, Input, Resource},
};

use crate::render::EguiRender;

/// plugin for building and rendering egui ui elements
///
/// required for using egui
pub struct EguiPlugin;

impl Plugin for EguiPlugin {
    fn setup(&self, app: &mut maple_app::App<maple_app::Init>) {
        app.context_mut().insert_resource(EguiResource {
            context: Context::default(),
            full_output: None,
        });

        app.renderer_mut()
            .graph()
            .setup_and_add_node::<EguiRender>();
    }

    fn update(&self, app: &mut maple_app::App<maple_app::Running>) {
        // let mut ctx = egui::Context::default();

        // // Game loop:
        // loop {
        //     let raw_input: egui::RawInput = gather_input();

        //     let full_output = ctx.run_ui(raw_input, |ui| {
        //         egui::CentralPanel::default().show(ui, |ui| {
        //             ui.label("Hello world!");
        //             if ui.button("Click me").clicked() {
        //                 // take some action here
        //             }
        //         });
        //     });
        //     handle_platform_output(full_output.platform_output);
        //     let clipped_primitives =
        //         ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
        //     paint(full_output.textures_delta, clipped_primitives);
        // }

        let input = crate::input::input_to_egui_raw_input(
            &app.context().get_resource::<Input>(),
            app.context().get_resource::<Frame>().elapsed.as_secs_f64(),
            true,
        );

        let ctx = app
            .context()
            .get_resource_mut::<EguiResource>()
            .context
            .clone();

        ctx.begin_pass(input);

        app.context().emit(EguiUpdate(ctx.clone()));

        let output = ctx.end_pass();

        app.context().get_resource_mut::<EguiResource>().full_output = Some(output);
    }
}

/// the egui resource containing the context and output
pub struct EguiResource {
    pub context: Context,
    pub full_output: Option<FullOutput>,
}

impl Resource for EguiResource {}

/// the update event for drawing egui ui
///
/// the [`maple_engine::components::EventCtx`] can be used directly as the [`Context`] for egui
pub struct EguiUpdate(pub Context);

impl Deref for EguiUpdate {
    type Target = Context;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl EventLabel for EguiUpdate {}

pub trait IntoColor {
    fn into_color(self) -> Color;
}

pub trait FromColor {
    fn from_color(color: Color) -> Self;
}

impl IntoColor for Color32 {
    fn into_color(self) -> Color {
        Color::from_8bit_rgba(self.r(), self.g(), self.b(), self.a())
    }
}

impl FromColor for Color32 {
    fn from_color(color: Color) -> Self {
        Color32::from_rgba_unmultiplied(
            (color.r.clamp(0.0, 1.0) * 255.0).round() as u8,
            (color.g.clamp(0.0, 1.0) * 255.0).round() as u8,
            (color.b.clamp(0.0, 1.0) * 255.0).round() as u8,
            (color.a.clamp(0.0, 1.0) * 255.0).round() as u8,
        )
    }
}
