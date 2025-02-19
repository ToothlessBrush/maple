//! UI node for the game context.
//!
//! UI nodes are nodes that are used to render UI elements on the screen. this engine uses egui for rendering UI elements. see the egui documentation for more information.
//!
//! # Usage
//! egui works by defining a closure that takes the egui context and the game context. this closure is then called every frame to render the UI.
//!
//! # Example
//! ```rust
//! use quaturn::game_context::nodes::ui::UI;
//! use quaturn::game_context::GameContext;
//! use quaturn::Engine;
//! use nalgebra_glm as glm;
//! use quaturn::egui;
//!
//! let mut engine = Engine::init("example", 800, 600);
//!
//! engine.context.nodes.add("ui", UI::init(&mut engine.context.window)).define_ui(|ctx, game_context| {
//!     egui::Window::new("Hello world").show(ctx, |ui| {
//!         
//!         ui.label("Hello world!");
//!         
//!         if ui.button("Click me").clicked() {
//!             println!("Button clicked!");
//!         }
//!     });
//! });
//!
//! //engine.begin();
//! ```

use egui_backend::egui;
use egui_backend::glfw;
use egui_gl_glfw as egui_backend;

use crate::components::NodeTransform;

use super::node_builder::NodeBuilder;

use std::sync::{Arc, Mutex};

use crate::context::node_manager::{Node, NodeManager};
use crate::context::GameContext;
use crate::renderer::Renderer;

/// UI node for defining UI elements in the game.
#[derive(Clone)]
pub struct UI {
    ctx: egui::Context,
    painter: Arc<Mutex<egui_backend::Painter>>,
    input: Arc<Mutex<egui_backend::EguiInputState>>,

    /// The transform of the node. while ui doesnt have a transform, it is still needed for the node system.
    pub transform: NodeTransform,
    /// The children of the node.
    pub children: NodeManager,
    native_pixels_per_point: f32,

    ui_window: Option<Arc<Mutex<dyn FnMut(&egui::Context, &mut GameContext)>>>,
}

impl Node for UI {
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }

    fn get_children(&self) -> &NodeManager {
        &self.children
    }

    fn get_children_mut(&mut self) -> &mut NodeManager {
        &mut self.children
    }
}

impl UI {
    /// Initializes a new UI node.
    ///
    /// # Arguments
    /// - `window` - The window to render the UI on.
    ///
    /// # Returns
    /// The new UI node.
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
            painter: Arc::new(Mutex::new(painter)),
            input: Arc::new(Mutex::new(input)),

            transform: NodeTransform::default(),
            children: NodeManager::new(),

            native_pixels_per_point,

            ui_window: None,
        }
    }

    pub fn update(&mut self, context: &mut GameContext) {
        // Lock the input to handle events
        if let Ok(mut input) = self.input.lock() {
            for (_, event) in context.input.events.iter() {
                // Clone the event because we need to use it multiple times
                egui_backend::handle_event(event.clone(), &mut *input);
            }

            // Update time and prepare the frame
            input.input.time = Some(context.frame.start_time.elapsed().as_secs_f64());
            self.ctx.begin_frame(input.input.take());
            input.pixels_per_point = self.native_pixels_per_point;
        } else {
            eprintln!("Failed to lock input for update");
        }
    }

    pub fn define_ui<F>(&mut self, ui_window: F) -> &mut UI
    where
        F: FnMut(&egui::Context, &mut GameContext) + 'static,
    {
        // Define the UI window by storing the closure in the Option
        self.ui_window = Some(Arc::new(Mutex::new(ui_window)));
        self
    }

    pub fn render(&mut self, context: &mut GameContext) {
        Renderer::ui_mode(true);

        // Check if a UI window definition exists and call the closure
        if let Some(ui_window) = &mut self.ui_window {
            if let Ok(mut ui_window) = ui_window.lock() {
                ui_window(&self.ctx, context);
            }
        }

        // End the frame and retrieve the output
        let egui::FullOutput {
            platform_output,
            textures_delta,
            shapes,
            pixels_per_point,
            viewport_output: _,
        } = self.ctx.end_frame();

        // Handle copied text from the UI (if any)
        if let Ok(mut input) = self.input.lock() {
            if !platform_output.copied_text.is_empty() {
                egui_backend::copy_to_clipboard(&mut *input, platform_output.copied_text);
            }
        } else {
            eprintln!("Failed to lock input for clipboard copy");
        }

        // Tessellate the shapes for rendering
        let clipped_shapes = self.ctx.tessellate(shapes, pixels_per_point);

        // Paint the shapes with the current painter
        if let Ok(mut painter) = self.painter.lock() {
            painter.paint_and_update_textures(1.0, &clipped_shapes, &textures_delta);
        } else {
            eprintln!("Failed to lock painter for rendering");
        }

        Renderer::ui_mode(false);
    }
}

pub trait UIBuilder {
    fn ui_component<F>(&mut self, ui_window: F) -> &mut Self
    where
        F: FnMut(&egui::Context, &mut GameContext) + 'static;
}

impl UIBuilder for NodeBuilder<UI> {
    fn ui_component<F>(&mut self, ui_window: F) -> &mut Self
    where
        F: FnMut(&egui::Context, &mut GameContext) + 'static,
    {
        self.node.define_ui(ui_window);
        self
    }
}
