//! the shader module contains the Shader struct, which is used to compile and manage shaders in the OpenGL pipeline
use colored::*;
use nalgebra_glm as glm; // Importing the nalgebra_glm crate for mathematical operations

pub enum Uniforms {
    ModelMatrix,
    ViewMatrix,
    ProjectionMatrix,
    LightPosition,
    LightColor,
    LightIntensity,
    EyePosition,
    MaterialColor,
    MaterialSpecularColor,
    MaterialShininess,
}

impl Uniforms {
    pub fn get_uniform_name(&self) -> String {
        match self {
            Uniforms::ModelMatrix => "u_modelMatrix".to_string(),
            Uniforms::ViewMatrix => "u_viewMatrix".to_string(),
            Uniforms::ProjectionMatrix => "u_projectionMatrix".to_string(),
            Uniforms::LightPosition => "u_lightPosition".to_string(),
            Uniforms::LightColor => "u_lightColor".to_string(),
            Uniforms::LightIntensity => "u_lightIntensity".to_string(),
            Uniforms::EyePosition => "u_eyePosition".to_string(),
            Uniforms::MaterialColor => "u_materialColor".to_string(),
            Uniforms::MaterialSpecularColor => "u_materialSpecularColor".to_string(),
            Uniforms::MaterialShininess => "u_materialShininess".to_string(),
        }
    }
}

/// The Shader struct is used to compile and manage shaders in the OpenGL pipeline
#[derive(Clone, Debug, Default)]
pub struct Shader {
    m_renderer_id: u32,
    m_uniform_location_cache: std::collections::HashMap<String, i32>,
}

// impl Default for Shader {
//     fn default() -> Self {
//         Shader::from_slice(
//             include_str!("../../res/shaders/default/default.vert"),
//             include_str!("../../res/shaders/default/default.frag"),
//             None,
//         )
//     }

// }

impl Shader {
    pub fn use_default() -> Self {
        Shader::from_slice(
            include_str!("../../res/shaders/default/default.vert"),
            include_str!("../../res/shaders/default/default.frag"),
            None,
        )
    }

    /// Creates a new shader object, optionally with a geometry shader
    ///
    /// # Arguments
    /// - `vertex_path` - The path to the vertex shader file
    /// - `fragment_path` - The path to the fragment shader file
    /// - `geometry_path` - The path to the geometry shader file (optional)
    pub fn new(vertex_path: &str, fragment_path: &str, geometry_path: Option<&str>) -> Shader {
        //println!("Compiling shader {:?}... ", vertex_path);
        let vertex_shader =
            std::fs::read_to_string(vertex_path).expect("Failed to read vertex shader file");
        //println!("Compiling shader {:?}... ", fragment_path);
        let fragment_shader =
            std::fs::read_to_string(fragment_path).expect("Failed to read fragment shader file");

        let geometry_shader = geometry_path.map(|path| {
            std::fs::read_to_string(path).expect("Failed to read geometry shader file")
        });

        Shader {
            m_renderer_id: Self::create_shader(
                &vertex_shader,
                &fragment_shader,
                geometry_shader.as_deref(),
            ),
            m_uniform_location_cache: std::collections::HashMap::new(),
        }
    }

    /// Creates a new shader object from literal shader code, optionally with a geometry shader
    ///
    /// # Arguments
    /// - `vertex` - The source code for the vertex shader
    /// - `fragment` - The source code for the fragment shader
    /// - `geometry` - The source code for the geometry shader (optional)
    pub fn from_slice(vertex: &str, fragment: &str, geometry: Option<&str>) -> Shader {
        Shader {
            m_renderer_id: Self::create_shader(vertex, fragment, geometry),
            m_uniform_location_cache: std::collections::HashMap::new(),
        }
    }

    /// Compiles and links shaders, including an optional geometry shader
    ///
    /// # Arguments
    /// - `vertex_shader` - The source code for the vertex shader
    /// - `fragment_shader` - The source code for the fragment shader
    /// - `geometry_shader` - The source code for the geometry shader (optional)
    fn create_shader(
        vertex_shader: &str,
        fragment_shader: &str,
        geometry_shader: Option<&str>,
    ) -> u32 {
        let program = unsafe { gl::CreateProgram() };
        let vs = Self::compile_shader(gl::VERTEX_SHADER, vertex_shader);
        let fs = Self::compile_shader(gl::FRAGMENT_SHADER, fragment_shader);

        unsafe {
            gl::AttachShader(program, vs);
            gl::AttachShader(program, fs);

            if let Some(gs_src) = geometry_shader {
                let gs = Self::compile_shader(gl::GEOMETRY_SHADER, gs_src);
                gl::AttachShader(program, gs);
                gl::DeleteShader(gs); // Clean up after attaching
            }

            gl::LinkProgram(program);
            gl::ValidateProgram(program);

            gl::DeleteShader(vs);
            gl::DeleteShader(fs);
        }

        program
    }

    /// Compiles individual shader stages
    ///
    /// # Arguments
    /// - `type_` - The type of shader to compile
    /// - `source` - The source code for the shader
    fn compile_shader(type_: u32, source: &str) -> u32 {
        let id = unsafe { gl::CreateShader(type_) };
        let c_str = std::ffi::CString::new(source).unwrap();

        unsafe {
            gl::ShaderSource(id, 1, &c_str.as_ptr(), std::ptr::null());
            gl::CompileShader(id);

            let mut result = gl::FALSE as i32;
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut result);
            if result == gl::FALSE as i32 {
                let mut length = 0;
                gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut length);
                let mut message = vec![0; length as usize];
                gl::GetShaderInfoLog(
                    id,
                    length,
                    std::ptr::null_mut(),
                    message.as_mut_ptr() as *mut i8,
                );
                println!(
                    "Failed to compile {:?} shader!",
                    match type_ {
                        gl::VERTEX_SHADER => "Vertex",
                        gl::FRAGMENT_SHADER => "Fragment",
                        gl::GEOMETRY_SHADER => "Geometry",
                        _ => "Unknown",
                    }
                );
                println!(
                    "{}",
                    std::str::from_utf8(&message).expect("Shader info log is not valid UTF-8")
                );
                gl::DeleteShader(id);
                return 0;
            }
        }

        id
    }

    /// Binds the shader for use in the OpenGL pipeline
    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.m_renderer_id);
        }
    }

    /// Unbinds the shader
    pub fn unbind(&self) {
        unsafe {
            gl::UseProgram(0);
        }
    }

    /// sets a unform for type T: Uniform
    ///
    /// this also binds the shader you set the uniform too
    ///
    /// # Arguements
    /// - `name` - the name of the unform
    /// - `value` - the value to set the uniform too
    pub fn set_uniform<T>(&mut self, name: &str, value: T)
    where
        T: Uniform,
    {
        self.bind();
        value.set_uniform(self.get_uniform_location(name));
    }

    /// Get the location of a uniform in the shader
    ///
    /// this function also caches the location of the uniform to avoid querying the gpu for the location
    ///
    /// # Arguments
    /// - `name` - the name of the uniform to get the location of
    ///
    /// # Returns
    /// the location of the uniform
    pub fn get_uniform_location(&mut self, name: &str) -> i32 {
        //get from cache since gpu -> cpu is forbidden by the computer gods
        if self.m_uniform_location_cache.contains_key(name) {
            return self.m_uniform_location_cache[name];
        }

        //get the location of the uniform if not in the cache
        let c_str = std::ffi::CString::new(name).unwrap();
        let location = unsafe {
            let location = gl::GetUniformLocation(self.m_renderer_id, c_str.as_ptr());
            if location == -1 {
                println!(
                    "{}",
                    format!("Warning: uniform '{:?}' doesn't exist!", name).yellow()
                );
            }
            location
        };

        self.m_uniform_location_cache
            .insert(name.to_string(), location);
        location
    }
}

/// a type implementing this has a equivelent type in glsl
pub trait Uniform {
    /// calls the gl::unform for the equivelent glsl type
    fn set_uniform(&self, location: i32);
}

impl Uniform for i32 {
    fn set_uniform(&self, location: i32) {
        unsafe {
            gl::Uniform1i(location, *self);
        }
    }
}

impl Uniform for f32 {
    fn set_uniform(&self, location: i32) {
        unsafe {
            gl::Uniform1f(location, *self);
        }
    }
}

impl Uniform for glm::Vec2 {
    fn set_uniform(&self, location: i32) {
        unsafe {
            gl::Uniform2f(location, self.x, self.y);
        }
    }
}

impl Uniform for glm::Vec3 {
    fn set_uniform(&self, location: i32) {
        unsafe {
            gl::Uniform3f(location, self.x, self.y, self.z);
        }
    }
}

impl Uniform for glm::Vec4 {
    fn set_uniform(&self, location: i32) {
        unsafe {
            gl::Uniform4f(location, self.x, self.y, self.z, self.w);
        }
    }
}

impl Uniform for glm::Mat4 {
    fn set_uniform(&self, location: i32) {
        unsafe {
            gl::UniformMatrix4fv(location, 1, gl::FALSE, self.as_ptr());
        }
    }
}

impl Uniform for bool {
    fn set_uniform(&self, location: i32) {
        unsafe {
            gl::Uniform1i(location, *self as i32);
        }
    }
}

impl Uniform for &[f32] {
    fn set_uniform(&self, location: i32) {
        unsafe {
            gl::Uniform1fv(location, self.len() as i32, self.as_ptr());
        }
    }
}

impl Uniform for &[i32] {
    fn set_uniform(&self, location: i32) {
        unsafe {
            gl::Uniform1iv(location, self.len() as i32, self.as_ptr());
        }
    }
}

impl Uniform for &[glm::Mat4] {
    fn set_uniform(&self, location: i32) {
        unsafe {
            if self.is_empty() {
                eprintln!("Tried to set array uniform to empty array!");
                return;
            }

            gl::UniformMatrix4fv(
                location,
                self.len() as i32,
                gl::FALSE,
                (*self.get_unchecked(0)).as_ptr(),
            );
        }
    }
}

impl Uniform for glm::Mat3 {
    fn set_uniform(&self, location: i32) {
        unsafe {
            gl::UniformMatrix3fv(location, 1, gl::FALSE, self.as_ptr());
        }
    }
}

impl Uniform for glm::Mat2 {
    fn set_uniform(&self, location: i32) {
        unsafe {
            gl::UniformMatrix2fv(location, 1, gl::FALSE, self.as_ptr());
        }
    }
}

impl Uniform for f64 {
    fn set_uniform(&self, location: i32) {
        unsafe {
            gl::Uniform1f(location, *self as f32);
        }
    }
}
