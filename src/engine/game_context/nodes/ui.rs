use egui_backend::egui;
use egui_backend::glfw;
use egui_gl_glfw as egui_backend;

use nalgebra_glm as glm;

use crate::engine::game_context::node_manager::Node;
use crate::engine::game_context::GameContext;
use crate::engine::renderer::Renderer;

pub struct UI {
    ctx: egui::Context,
    painter: egui_backend::Painter,
    input: egui_backend::EguiInputState,

    native_pixels_per_point: f32,

    ui_window: Option<Box<dyn FnMut(&egui::Context, &mut GameContext)>>,

    ready_callback: Option<Box<dyn FnMut(&mut Self)>>,
    behavior_callback: Option<Box<dyn FnMut(&mut Self, &mut GameContext)>>,
}

impl Node for UI {
    type Transform = ();

    fn get_model_matrix(&self) -> glm::Mat4 {
        glm::identity()
    }

    fn get_transform(&self) -> &Self::Transform {
        &()
    }
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

            ready_callback: None,
            behavior_callback: None,
        }
    }

    pub fn update(&mut self, context: &mut GameContext) {
        for (_, event) in context.input.events.iter() {
            //clone the event instead of dereferencing it since we need to use it multiple times
            egui_backend::handle_event(event.clone(), &mut self.input);
        }
        self.input.input.time = Some(context.frame.start_time.elapsed().as_secs_f64());
        self.ctx.begin_frame(self.input.input.take());
        self.input.pixels_per_point = self.native_pixels_per_point;
    }

    pub fn define_ui<F>(&mut self, ui_window: F) -> &mut UI
    where
        F: FnMut(&egui::Context, &mut GameContext) + 'static,
    {
        self.ui_window = Some(Box::new(ui_window));
        self
    }

    pub fn render(&mut self, context: &mut GameContext) {
        Renderer::ui_mode(true);

        if let Some(ui_window) = &mut self.ui_window {
            ui_window(&self.ctx, context);
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

        Renderer::ui_mode(false);
    }
}
