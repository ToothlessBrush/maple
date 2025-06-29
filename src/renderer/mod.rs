//! the renderer module is responsible for all the rendering related tasks including opengl initialization, shader compilation, textures, shadows, etc...
use crate::context::GameContext;
use crate::gl;
use crate::nodes::Camera3D;
use crate::nodes::Model;
use crate::nodes::directional_light::DirectionalLightBufferData;
use crate::nodes::node::Drawable;
use crate::nodes::point_light::PointLightBufferData;
use crate::render_passes::RenderPass;
use crate::utils::color::BLACK;
use buffers::storage_buffer::StorageBuffer;
use depth_cube_map_array::DepthCubeMapArray;
use depth_map_array::DepthMapArray;
use egui_backend::glfw;
use egui_gl_glfw as egui_backend;
use nalgebra_glm::Vec4;
use shader::Shader;

use crate::components::mesh::AlphaMode;
use crate::components::mesh::Mesh;

use std::ffi::CStr;

pub mod buffers;
pub mod debug_message_callback;
pub mod depth_cube_map;
pub mod depth_cube_map_array;
pub mod depth_map_array;
pub mod shader;
pub mod shadow_map;
pub mod texture;

use colored::*;

const MAX_DIRECT_LIGHTS: usize = 10;
const MAX_POINT_LIGHTS: usize = 10;

/// contains state info about the scene
pub struct SceneState {
    /// bias offset for shadows
    pub bias_offset: f32,
    /// bias factor for shadows
    pub bias_factor: f32,
    /// the amount of ambient light in a scene
    pub ambient_light: f32,
}

impl Default for SceneState {
    fn default() -> Self {
        Self {
            bias_offset: 0.000006, // these produced that best shadows after testing
            bias_factor: 0.000200,
            ambient_light: 0.02,
        }
    }
}

/// Renderer struct contains a bunch of static methods to initialize and render the scene
pub struct Renderer {
    passes: Vec<Box<dyn RenderPass>>,
    /// default shader of the scene used during the main pass
    pub default_shader: Shader,
    /// the shadows for point lights
    pub shadow_cube_maps: DepthCubeMapArray,
    /// the shadows for directional lights
    pub shadow_maps: DepthMapArray,
    /// the current state of the scene
    pub scene_state: SceneState,
    /// ssbo for directional lights
    pub direct_light_buffer: StorageBuffer,
    /// ssbo for point lights
    pub point_light_buffer: StorageBuffer,
    /// clear color of the scene
    pub clear_color: Vec4,
}

impl Renderer {
    /// initialize the renderer and opengl
    pub fn init() -> Self {
        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::DebugMessageCallback(
                Some(debug_message_callback::debug_message_callback),
                std::ptr::null(),
            );

            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);

            gl::Enable(gl::MULTISAMPLE);

            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
            gl::FrontFace(gl::CCW);

            //enable on draw call
            //gl::Enable(gl::BLEND);

            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            let x = gl::GetString(gl::VERSION);
            let cstr = CStr::from_ptr(x as *const i8);
            println!("{}", format!("Using OpenGL Version: {:?}", cstr).cyan());
        }

        Self {
            passes: Vec::new(),
            default_shader: Shader::use_default(),
            scene_state: SceneState::default(),
            clear_color: BLACK.into(),
            shadow_maps: DepthMapArray::gen_map(
                4096,
                4096,
                MAX_DIRECT_LIGHTS,
                Shader::from_slice(
                    include_str!("../../res/shaders/depthShader/depthShader.vert"),
                    include_str!("../../res/shaders/depthShader/depthShader.frag"),
                    Some(include_str!(
                        "../../res/shaders/depthShader/depthShader.geom"
                    )),
                ),
            ),
            shadow_cube_maps: DepthCubeMapArray::gen_map(
                1024,
                1024,
                MAX_POINT_LIGHTS,
                Shader::from_slice(
                    include_str!("../../res/shaders/cubeDepthShader/cubeDepthShader.vert"),
                    include_str!("../../res/shaders/cubeDepthShader/cubeDepthShader.frag"),
                    Some(include_str!(
                        "../../res/shaders/cubeDepthShader/cubeDepthShader.geom"
                    )),
                ),
            ),
            direct_light_buffer: StorageBuffer::new(
                (MAX_DIRECT_LIGHTS * std::mem::size_of::<DirectionalLightBufferData>()) as isize,
            ),
            point_light_buffer: StorageBuffer::new(
                (MAX_POINT_LIGHTS * std::mem::size_of::<PointLightBufferData>()) as isize,
            ),
        }
    }

    /// adds a render pass to the render step
    ///
    /// it renders the passes in the order you add them
    pub fn add_pass<T: RenderPass + 'static>(&mut self, pass: T) {
        self.passes.push(Box::new(pass))
    }

    /// renderers the scene
    pub fn render(&mut self, context: &GameContext) {
        let camera_path = context.active_camera_path.clone();
        let Some(camera) = traverse_camera_path(context, camera_path) else {
            return;
        };

        let models = context.scene.collect_items::<Model>();

        let drawables: Vec<&dyn Drawable> =
            models.iter().map(|model| *model as &dyn Drawable).collect();

        let mut passes = std::mem::take(&mut self.passes);

        for pass in &mut passes {
            pass.render(self, context, &drawables, camera);
        }

        self.passes = passes
    }

    /// overrides the default shader used to render the mainpass
    pub fn override_default_shader(&mut self, shader: Shader) {
        self.default_shader = shader
    }

    /// add the context to the window
    ///
    /// # Arguments
    /// - `window` - the window to add the context to
    pub fn context(window: &mut glfw::Window) {
        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
    }

    /// clear the screen
    pub fn clear() {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    /// set the clear color
    ///
    /// # Arguments
    /// - `color` - the color to clear the screen with (rgba)
    pub fn set_clear_color(&mut self, color: impl Into<crate::math::Vec4>) {
        unsafe {
            let color: crate::math::Vec4 = color.into();
            if color != self.clear_color {
                gl::ClearColor(color.x, color.y, color.z, color.w);
                self.clear_color = color
            }
        }
    }

    /// set the viewport size
    ///
    /// # Arguments
    /// - `width` - the width of the viewport
    /// - `height` - the height of the viewport
    pub fn viewport(width: i32, height: i32) {
        unsafe {
            gl::Viewport(0, 0, width, height);
        }
    }

    /// draw a mesh
    ///
    /// # Arguments
    /// - `mesh` - the mesh to draw
    pub fn draw(mesh: &Mesh) {
        if mesh.material_properties.double_sided {
            unsafe {
                gl::Disable(gl::CULL_FACE);
            }
        }
        match mesh.material_properties.alpha_mode {
            AlphaMode::Opaque => unsafe {
                gl::Disable(gl::BLEND);
                gl::DepthMask(gl::TRUE); // Enable depth writing for opaque objects
            },
            AlphaMode::Blend => unsafe {
                //println!("blending");
                gl::Enable(gl::BLEND);
                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA); // Typical blending setup
                gl::DepthMask(gl::FALSE);
            },
            AlphaMode::Mask => unsafe {
                gl::Disable(gl::BLEND);
                gl::DepthMask(gl::TRUE); // Enable depth writing for masked objects
            },
        }

        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                mesh.indices.len() as i32,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        }

        if mesh.material_properties.double_sided {
            unsafe {
                gl::Enable(gl::CULL_FACE);
            }
        }

        // Reset the blending and depth mask
        if mesh.material_properties.alpha_mode == AlphaMode::Blend {
            unsafe {
                gl::Disable(gl::BLEND);
                gl::DepthMask(gl::TRUE);
            }
        }
    }

    /// clears the depth buffer
    pub fn clear_depth_buffer() {
        unsafe {
            gl::Clear(gl::DEPTH_BUFFER_BIT);
        }
    }

    /// set the renderer to ui mode to render the ui
    pub fn ui_mode(enabled: bool) {
        if enabled {
            unsafe {
                gl::Disable(gl::CULL_FACE);
                gl::Disable(gl::DEPTH_TEST);
            }
        } else {
            unsafe {
                gl::Enable(gl::CULL_FACE);
                gl::Enable(gl::DEPTH_TEST);
                gl::BindTexture(gl::TEXTURE_2D, 0); // need to unbind the texture that ui uses
            }
        }
    }
}

/// we store the active camera path so in order to get it we need to traverse it
fn traverse_camera_path(context: &GameContext, camera_path: Vec<String>) -> Option<&Camera3D> {
    // Early return if path is empty
    if camera_path.is_empty() {
        return None;
    }

    let mut current_node = context.scene.get_dyn(&camera_path[0])?;

    for index in &camera_path[1..] {
        current_node = current_node.get_children().get_dyn(index)?;
    }

    current_node.as_any().downcast_ref::<Camera3D>()
}
