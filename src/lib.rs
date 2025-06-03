#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
use std::error::Error;

// I wish I used glow ngl
#[allow(warnings)]
pub(crate) mod gl {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use components::Event;
use context::fps_manager::FrameInfo;
use context::scene::Scene;
use egui_gl_glfw::glfw::Cursor;
use egui_gl_glfw::glfw::WindowMode;
pub use nalgebra_glm as math;

//re-exporting the engine module
pub use egui_gl_glfw::egui;
pub use egui_gl_glfw::glfw;

use egui_gl_glfw::glfw::Context;
use nodes::DirectionalLight;
use render_passes::cube_shadow_pass::CubeShadowPass;
use render_passes::{main_pass::MainPass, shadow_pass::ShadowPass};
pub use utils::config::EngineConfig;

use crate::nodes::{Model, PointLight, UI};
use renderer::Renderer;
use renderer::shader::Shader;

pub mod components;
pub mod context;
pub mod nodes;
pub mod render_passes;
pub mod renderer;
pub mod utils;

use context::GameContext;

/// Represents the main game engine.
///
/// The Enigne is responsible for managing the game loop and rendering the scene.
pub struct Engine {
    /// The game context such as the frame, input, nodes, and shaders.
    pub context: GameContext,
    /// configuration of the engine
    pub config: EngineConfig,
    /// renderer of the engine
    renderer: Renderer,
}

/// The number of samples for anti-aliasing.
const SAMPLES: u32 = 8;

impl Engine {
    /// Initializes the game engine.
    ///
    /// # Arguments
    /// - `config` - initial config of the engine
    ///
    /// # Returns
    /// A new instance of the Engine. or an error
    ///
    /// # Example
    /// ```rust
    /// use maple::Engine;
    /// let mut engine = Engine::init(EngineConfig {
    ///     ..Default::default()
    /// });
    /// ```
    pub fn init(config: EngineConfig) -> Result<Engine, Box<dyn Error>> {
        use glfw::fail_on_errors;
        let mut glfw = glfw::init(fail_on_errors!())?;
        glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));
        glfw.window_hint(glfw::WindowHint::DoubleBuffer(true));
        glfw.window_hint(glfw::WindowHint::Resizable(false));
        glfw.window_hint(glfw::WindowHint::Samples(Some(SAMPLES)));
        //glfw.window_hint(glfw::WindowHint::RefreshRate(Some(60)));

        let (mut window, events) = match config.window_mode {
            utils::config::WindowMode::Windowed => glfw
                .create_window(
                    config.resolution.width,
                    config.resolution.height,
                    &config.window_title,
                    WindowMode::Windowed,
                )
                .ok_or("failed to create window")?,
            utils::config::WindowMode::FullScreen => glfw
                .with_primary_monitor(|g, monitor| {
                    let mut width = config.resolution.width;
                    let mut height = config.resolution.height;

                    if let Some(monitor) = &monitor {
                        if let Some(vid_mode) = monitor.get_video_mode() {
                            width = vid_mode.width;
                            height = vid_mode.height;
                        }
                    }

                    g.create_window(
                        width,
                        height,
                        &config.window_title,
                        monitor.map_or(WindowMode::Windowed, |m| WindowMode::FullScreen(m)),
                    )
                })
                .ok_or("failed to create window")?,
            _ => glfw
                .create_window(
                    config.resolution.width,
                    config.resolution.height,
                    &config.window_title,
                    WindowMode::Windowed,
                )
                .ok_or("failed to create window")?,
        };

        //set up input polling
        window.set_key_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_mouse_button_polling(true);
        window.set_scroll_polling(true);
        window.make_current();

        window.set_cursor(Some(Cursor::standard(glfw::StandardCursor::IBeam)));

        if glfw.supports_raw_motion() {
            window.set_raw_mouse_motion(true);
        }

        //load grahpics api
        Renderer::context(&mut window);

        glfw.set_swap_interval(glfw::SwapInterval::None);

        let renderer = Renderer::init();

        Ok(Engine {
            context: GameContext::new(events, glfw, window),
            //shadow_map: None,
            config,

            renderer,
        })
    }

    /// load a scene into the games Context
    ///
    /// # Arguments
    /// - `scene`: the scene to be added to the context's Scene
    pub fn load_scene(&mut self, scene: Scene) {
        self.context.scene.load(scene);
    }

    /// starts the gamme/render loop.
    ///
    /// this function is responsible for rendering the scene and updating the game context.
    ///
    /// # Example
    /// ```rust
    /// use maple::Engine;
    /// let mut engine = Engine::init("My Game", 800, 600);
    ///
    /// //set up the scene
    ///
    /// engine.begin();
    /// ```
    pub fn begin(&mut self) -> Result<(), Box<dyn Error>> {
        self.renderer.add_pass(ShadowPass);
        self.renderer.add_pass(CubeShadowPass);
        self.renderer.add_pass(MainPass);

        self.context.emit(Event::Ready);

        if self.context.scene.active_shader.is_empty() {
            eprintln!("INFO: No shader override using default shader");
            self.context
                .scene
                .add_shader("default", Shader::use_default());
        }

        self.update_ui();

        //render loop
        self.render_loop()
    }

    /// The main render loop.
    /// This function is responsible for rendering the scene and updating the game context.
    /// It is called by the `begin` function.
    fn render_loop(&mut self) -> Result<(), Box<dyn Error>> {
        while !self.context.window.should_close() {
            let mut frame_info = FrameInfo::default();

            time!(&mut frame_info.clear_time, { Renderer::clear() });

            time!(&mut frame_info.render_time, {
                self.renderer.render(&self.context)
            });

            time!(&mut frame_info.ui_pass_time, { self.render_ui_pass() });

            // update context while the gpu is rendering
            time!(&mut frame_info.context_update_time, {
                self.update_context()
            });

            time!(&mut frame_info.ui_update_time, { self.update_ui() });

            time!(&mut frame_info.event_emit_time, {
                self.context.emit(Event::Update)
            });

            // swap buffers
            time!(&mut frame_info.swap_buffers_time, {
                self.context.window.swap_buffers()
            });

            self.context.frame.frame_info = frame_info;
        }
        Ok(())
    }

    /// sets the window set_title
    ///
    /// # Arguements
    /// - 'title' - the title
    pub fn set_window_title(&mut self, title: &str) {
        self.context.window.set_title(title);
    }

    fn update_context(&mut self) {
        let context = &mut self.context;

        context.frame.update();

        context.input.update();
    }

    fn update_ui(&mut self) {
        let nodes = self.context.scene.get_iter::<UI>();

        //map nodes to raw pointer to borrowed twice
        let nodes: Vec<*mut UI> = nodes.map(|node| node as *const UI as *mut UI).collect();

        for ui in nodes {
            unsafe {
                (*ui).update(&mut self.context);
            }
        }
    }

    fn render_ui_pass(&mut self) {
        let nodes = self.context.scene.get_iter::<UI>();

        //map nodes to raw pointer to borrowed twice
        let nodes: Vec<*mut UI> = nodes.map(|node| node as *const UI as *mut UI).collect();

        for ui in nodes {
            unsafe {
                // SAFETY: we are using raw pointers here because we guarantee
                // that the nodes vector will not be modified (no adding/removing nodes)
                // during this iteration instead that is needs to be handled through a queue system
                (*ui).render(&mut self.context)
            }
        }
    }
}

/// Converts a mutable reference to a Model to a raw pointer.
impl From<&'static mut Model> for *mut Model {
    fn from(model: &'static mut Model) -> Self {
        model as *mut Model
    }
}

impl From<&'static mut PointLight> for *mut PointLight {
    fn from(light: &'static mut PointLight) -> Self {
        light as *mut PointLight
    }
}

impl From<&'static mut DirectionalLight> for *mut DirectionalLight {
    fn from(value: &'static mut DirectionalLight) -> Self {
        value as *mut DirectionalLight
    }
}
