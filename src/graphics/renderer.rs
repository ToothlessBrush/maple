use std::ptr::null;

use super::buffers::index_buffer;
use super::buffers::vertex_array;
use super::camera::{Camera2D, Camera3D};
//use super::game_object;
use super::shader;

use colored::*;

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

    println!(
        "\n{}\nSource: {} Type: {} ID: {} Severity: {}\n",
        message.yellow(),
        source_str.yellow(),
        _type.to_string().yellow(),
        id.to_string().yellow(),
        severity_str.yellow(),
    );
}

enum Camera {
    Camera2D(Camera2D),
    Camera3D(Camera3D),
}

pub struct Renderer {
    pub camera: Camera3D,
}

impl Renderer {
    pub fn new(camera: Camera3D) -> Renderer {
        Renderer { camera }
    }

    /* ///adds an object to the renderer
    pub fn add_object(&mut self, object: game_object::GameObject) {
        self.objects.push(object);
    } */

    ///draws all objects in the renderer
    /* pub fn draw_objects(&self, shader: &mut shader::Shader) {
        for object in self.objects.iter() {
            let vp: glm::Mat4 = self.camera.get_vp(); //the view projection matrix from the camera
            let mvp: glm::Mat4 = vp * object.get_transform(); //the model view projection matrix
            shader.set_uniform_mat4f("u_MVP", &mvp); //set the mvp matrix in the shader
            object.draw(shader); //draw the object
        }
    } */
    //todo
    ///apply mvp to all objects then batch them togeather to reduce draw calls
    //pub fn batch_objects() {}

    ///draws a single object from va and ib with shader
    pub fn draw(
        &self,
        va: &vertex_array::VertexArray,
        ib: &index_buffer::IndexBuffer,
        shader: &shader::Shader,
    ) {
        shader.bind();
        va.bind();
        ib.bind();
        unsafe {
            //gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
            gl::DrawElements(
                gl::TRIANGLES,
                ib.get_count(),
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        }
    }

    // pub fn draw_object(
    //     &self,
    //     object: &super::game_object::GameObject,
    //     shader: &mut shader::Shader,
    // ) {
    //     shader.bind();
    //     object.get_va().bind();
    //     object.get_ib().bind();

    //     let model = object.get_transform();
    //     let vp = self.camera.get_vp_matrix();
    //     //let mvp = vp * model;
    //     shader.set_uniform_mat4f("u_VP", &vp);
    //     shader.set_uniform_mat4f("u_Model", &model);

    //     object.get_texture().bind(); //bind the texture to a texture unit on the gpu
    //     shader.set_uniform1i("u_Texture0", 0); //set the texture unit in the shader

    //     unsafe {
    //         //gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
    //         gl::DrawElements(
    //             gl::TRIANGLES,
    //             object.get_ib().get_count(),
    //             gl::UNSIGNED_INT,
    //             null(),
    //         );
    //     }
    // }

    /* pub fn draw_instanced(
        &self,
        va: &vertex_array::VertexArray,
        ib: &index_buffer::IndexBuffer,
        shader: &shader::Shader,
        count: i32,
    ) {
        shader.bind();
        va.bind();
        ib.bind();
        unsafe {
            //gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
            gl::DrawElementsInstanced(
                gl::TRIANGLES,
                ib.get_count(),
                gl::UNSIGNED_INT,
                std::ptr::null(),
                count,
            );
        }
    } */

    pub fn clear(&self, color: (f32, f32, f32, f32)) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::ClearColor(color.0, color.1, color.2, color.3);
        }
    }
}
