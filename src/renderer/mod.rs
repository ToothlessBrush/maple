//! te renderer module is responsible for all the rendering related tasks including opengl initialization, shader compilation, textures, shadows, etc...
use crate::context::GameContext;
use crate::gl;
use crate::nodes::Camera3D;
use crate::nodes::Model;
use crate::nodes::Node;
use crate::nodes::directional_light::DirectionalLightBufferData;
use crate::nodes::node::Drawable;
use crate::nodes::point_light::PointLightBufferData;
use crate::render_passes::RenderPass;
use buffers::storage_buffer::StorageBuffer;
use depth_cube_map_array::DepthCubeMapArray;
use depth_map_array::DepthMapArray;
use egui_backend::glfw;
use egui_gl_glfw as egui_backend;
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

pub struct SceneState {
    pub bias_offset: f32,
    pub bias_factor: f32,
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
    pub default_shader: Shader,
    pub shadow_cube_maps: DepthCubeMapArray,
    pub shadow_maps: DepthMapArray,
    pub scene_state: SceneState,

    pub direct_light_buffer: StorageBuffer,

    pub point_light_buffer: StorageBuffer,
}

impl Renderer {
    // initialize the renderer and opengl
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
    pub fn set_clear_color(color: impl Into<crate::math::Vec4>) {
        unsafe {
            let color: crate::math::Vec4 = color.into();
            gl::ClearColor(color.x, color.y, color.z, color.w);
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

/// traverses the scene and returns the nodes of a given type
fn collect_items<'a, T: Node>(node: &'a dyn Node, items: &mut Vec<&'a T>) {
    // Check if the current node matches the target type `N`
    if let Some(target) = node.as_any().downcast_ref::<T>() {
        // Use `unsafe` to extend the lifetime as static (assuming safe usage)
        items.push(target)
    }

    // Recursively collect items from children
    for child in node.get_children().get_all().values() {
        let child_node: &dyn Node = &**child;
        collect_items::<T>(child_node, items);
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
