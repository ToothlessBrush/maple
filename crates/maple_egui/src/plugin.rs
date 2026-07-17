use egui::{Context, FullOutput};
use maple_app::Plugin;
use maple_engine::prelude::{EventLabel, Frame, Input, ResMut, Resource};

use crate::render::EguiRender;

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

        app.context().emit(EguiUpdate { ctx: ctx.clone() });

        let output = ctx.end_pass();

        app.context().get_resource_mut::<EguiResource>().full_output = Some(output);
    }
}

pub struct EguiResource {
    pub context: Context,
    pub full_output: Option<FullOutput>,
}

impl Resource for EguiResource {}

pub struct EguiUpdate {
    pub ctx: Context,
}

impl EventLabel for EguiUpdate {}
