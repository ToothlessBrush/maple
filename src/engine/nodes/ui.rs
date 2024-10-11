use egui_backend::egui;
use egui_backend::glfw;
use egui_gl_glfw as egui_backend;

use super::super::utils::{fps_manager::FPSManager, input_manager::InputManager};

pub struct UI {
    ctx: egui::Context,
    painter: egui_backend::Painter,
    input: egui_backend::EguiInputState,

    native_pixels_per_point: f32,

    ui_window: Option<Box<dyn Fn()>>,
}

impl UI {
    pub fn init(window: &mut glfw::PWindow) -> UI {
        let (width, height) = window.get_framebuffer_size();
        let native_pixels_per_point = window.get_content_scale().0;

        let ctx = egui::Context::default();
        let painter = egui_backend::Painter::new(window);
        let input = egui_backend::EguiInputState::new(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::new(0.0, 0.0),
                    egui::Vec2::new(width as f32, height as f32),
                )),
                ..Default::default()
            },
            native_pixels_per_point,
        );

        UI {
            ctx,
            painter,
            input,

            native_pixels_per_point,

            ui_window: None,
        }
    }

    pub fn define_ui<F>(&mut self, ui_window: F)
    where
        F: Fn() + 'static,
    {
        self.ui_window = Some(Box::new(ui_window));
    }

    pub fn update(&mut self, fps_manager: &FPSManager, input_manager: &mut InputManager) {
        for (_, event) in glfw::flush_messages(&input_manager.events) {
            egui_backend::handle_event(event, &mut self.input);
        }
        self.input.input.time = Some(fps_manager.start_time.elapsed().as_secs_f64());
        self.ctx.begin_frame(self.input.input.take());
        self.input.pixels_per_point = self.native_pixels_per_point;
    }

    pub fn render(&mut self) {
        if let Some(ui_window) = &mut self.ui_window {
            ui_window();
        }

        let egui::FullOutput {
            platform_output,
            textures_delta,
            shapes,
            pixels_per_point,
            viewport_output: _,
        } = self.ctx.end_frame();

        if !platform_output.copied_text.is_empty() {
            egui_backend::copy_to_clipboard(&mut self.input, platform_output.copied_text);
        }

        let clipped_shapes = self.ctx.tessellate(shapes, pixels_per_point);
        self.painter
            .paint_and_update_textures(1.0, &clipped_shapes, &textures_delta);
    }
}
