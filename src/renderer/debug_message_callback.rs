//! Callback function for OpenGL debug messages
use crate::gl;

use colored::*;

/// called when opengl encounters an error
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
