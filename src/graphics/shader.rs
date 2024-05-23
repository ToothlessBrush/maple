pub struct Shader {
    m_renderer_id: u32,
    m_unfirom_location_cache: std::collections::HashMap<std::string::String, i32>,
}

impl Shader {
    /// creates a new shader object
    pub fn new(file_path: &str) -> Shader {
        let source: (std::string::String, std::string::String) = Self::parse_shader(file_path);
        Shader {
            m_renderer_id: Self::create_shader(&source.0, &source.1),
            m_unfirom_location_cache: std::collections::HashMap::new(),
        }
    }

    /// parses the shader files and returns the source code tuple
    fn parse_shader(file_path: &str) -> (std::string::String, std::string::String) {
        let mut fragment_shader = String::new();
        let mut vertex_shader = String::new();

        for file in std::fs::read_dir(file_path).unwrap() {
            let file = file.unwrap();
            match file.path().extension().unwrap().to_str().unwrap() {
                "frag" => fragment_shader = std::fs::read_to_string(file.path()).unwrap(),
                "vert" => vertex_shader = std::fs::read_to_string(file.path()).unwrap(),
                _ => {}
            }
        }

        (vertex_shader, fragment_shader)
    }

    /// compiles and binds shader programs
    fn create_shader(vertex_shader: &str, fragment_shader: &str) -> u32 {
        let program = unsafe { gl::CreateProgram() };
        let vs = Self::compile_shader(gl::VERTEX_SHADER, vertex_shader);
        let fs = Self::compile_shader(gl::FRAGMENT_SHADER, fragment_shader);

        unsafe {
            gl::AttachShader(program, vs);
            gl::AttachShader(program, fs);
            gl::LinkProgram(program);
            gl::ValidateProgram(program);

            gl::DeleteShader(vs);
            gl::DeleteShader(fs);
        }

        program
    }

    /// binds the shader program
    fn compile_shader(type_: u32, source: &str) -> u32 {
        println!(
            "Compiling shader: {:?} shader...",
            if type_ == gl::VERTEX_SHADER {
                "Vertex"
            } else {
                "Fragment"
            }
        );
        let id = unsafe { gl::CreateShader(type_) };
        let c_str = std::ffi::CString::new(source).unwrap();
        unsafe {
            gl::ShaderSource(id, 1, &c_str.as_ptr(), std::ptr::null());
            gl::CompileShader(id);
        }

        let mut result = gl::FALSE as i32;
        //get the status for shader error checking
        unsafe {
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut result);
            if result == gl::FALSE as i32 {
                let mut length = 0;
                gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut length);
                let mut message = Vec::with_capacity(length as usize);
                message.set_len(length as usize);
                gl::GetShaderInfoLog(
                    id,
                    length,
                    std::ptr::null_mut(),
                    message.as_mut_ptr() as *mut i8,
                );
                println!(
                    "Failed to compile {:?} shader!",
                    if type_ == gl::VERTEX_SHADER {
                        "Vertex"
                    } else {
                        "Fragment"
                    }
                );
                println!(
                    "{:?}",
                    std::str::from_utf8(&message).expect("Shader info log is not valid utf8")
                );
                gl::DeleteShader(id);
                return 0;
            }
        }
        return id;
    }

    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.m_renderer_id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::UseProgram(0);
        }
    }

    pub fn set_uniform4f(&mut self, name: &str, v0: f32, v1: f32, v2: f32, v3: f32) {
        unsafe {
            gl::Uniform4f(self.get_uniform_location(name), v0, v1, v2, v3);
        }
    }

    pub fn get_uniform_location(&mut self, name: &str) -> i32 {
        if self.m_unfirom_location_cache.contains_key(name) {
            return self.m_unfirom_location_cache[name];
        }

        let c_str = std::ffi::CString::new(name).unwrap();
        let location = unsafe {
            let location = gl::GetUniformLocation(self.m_renderer_id, c_str.as_ptr());
            if location == -1 {
                println!("Warning: uniform '{:?}' doesn't exist!", name);
            }
            location
        };

        self.m_unfirom_location_cache
            .insert(name.to_string(), location);
        location
    }
}
