#![allow(unused_variables)]
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
    println!(
        "GL CALLBACK: {} type = {}, severity = {}, message = {}",
        id, _type, severity, message
    );
}
