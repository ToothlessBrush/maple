//! the renderer module is responsible for all the rendering related tasks including opengl initialization, shader compilation, textures, shadows, etc...
use egui_backend::gl;
use egui_backend::glfw;
use egui_gl_glfw as egui_backend;

use crate::game_context::nodes::mesh::Mesh;

pub mod buffers;
pub mod shader;
pub mod shadow_map;
pub mod texture;

use colored::*;

/// Callback function for OpenGL debug messages
pub extern "system" fn debug_message_callback(
    source: gl::types::GLenum,
    _type: gl::types::GLenum,
    id: gl::types::GLuint,
    severity: gl::types::GLenum,
    length: gl::types::GLsizei,
    message: *const gl::types::GLchar,
    _user_param: *mut std::ffi::c_void,
) {
    let message = unsafe { std::slice::from_raw_parts(message as *const u8, length as usize) };
    let message = String::from_utf8_lossy(message);

    let source_str = match source {
        gl::DEBUG_SOURCE_API => "API",
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => "Window System",
        gl::DEBUG_SOURCE_SHADER_COMPILER => "Shader Compiler",
        gl::DEBUG_SOURCE_THIRD_PARTY => "Third Party",
        gl::DEBUG_SOURCE_APPLICATION => "Application",
        gl::DEBUG_SOURCE_OTHER => "Other",
        _ => "Unknown",
    };

    let severity_str = match severity {
        gl::DEBUG_SEVERITY_HIGH => "HIGH",
        gl::DEBUG_SEVERITY_MEDIUM => "MEDIUM",
        gl::DEBUG_SEVERITY_LOW => "LOW",
        gl::DEBUG_SEVERITY_NOTIFICATION => "NOTIFICATION",
        _ => "Unknown",
    };

    let _type = match _type {
        gl::DEBUG_TYPE_ERROR => "Error",
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "Deprecated Behavior",
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "Undefined Behavior",
        gl::DEBUG_TYPE_PORTABILITY => "Portability",
        gl::DEBUG_TYPE_PERFORMANCE => "Performance",
        gl::DEBUG_TYPE_MARKER => "Marker",
        gl::DEBUG_TYPE_PUSH_GROUP => "Push Group",
        gl::DEBUG_TYPE_POP_GROUP => "Pop Group",
        gl::DEBUG_TYPE_OTHER => "Other",
        _ => "Unknown",
    };

    if severity == gl::DEBUG_SEVERITY_HIGH {
        panic!(
            "\n{}\nSource: {}\nType: {}\nID: {}\nSeverity: {}\n",
            message.red(),
            source_str.red(),
            _type.to_string().red(),
            id.to_string().red(),
            severity_str.red()
        );
    }

    // println!(
    //     "\n{}\nSource: {} Type: {} ID: {} Severity: {}\n",
    //     message.yellow(),
    //     source_str.yellow(),
    //     _type.to_string().yellow(),
    //     id.to_string().yellow(),
    //     severity_str.yellow(),
    // );
}

/// Renderer struct contains a bunch of static methods to initialize and render the scene
pub struct Renderer {}

impl Renderer {
    /// initialize the renderer and opengl
    pub fn init() {
        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::DebugMessageCallback(Some(debug_message_callback), std::ptr::null());

            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);

            gl::Enable(gl::MULTISAMPLE);

            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
            gl::FrontFace(gl::CCW);

            //enable on draw call
            //gl::Enable(gl::BLEND);

            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
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
    pub fn set_clear_color(color: [f32; 4]) {
        unsafe {
            gl::ClearColor(color[0], color[1], color[2], color[3]);
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
        match mesh.material_properties.alpha_mode.as_str() {
            "OPAQUE" => unsafe {
                gl::Disable(gl::BLEND);
                gl::DepthMask(gl::TRUE); // Enable depth writing for opaque objects
            },
            "BLEND" => unsafe {
                gl::Enable(gl::BLEND);
                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA); // Typical blending setup
                gl::DepthMask(gl::TRUE);
            },
            "MASK" => unsafe {
                gl::Disable(gl::BLEND);
                gl::DepthMask(gl::TRUE); // Enable depth writing for masked objects
            },
            _ => unsafe {
                gl::Disable(gl::BLEND);
                gl::DepthMask(gl::TRUE);
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
            }
        }
    }
}
