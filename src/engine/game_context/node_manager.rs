use super::nodes::{camera::Camera3D, model::Model, ui::UI};

use crate::engine::renderer::shader::Shader;
use std::collections::HashMap;

pub struct NodeManager {
    pub models: HashMap<String, Model>,
    pub cameras: HashMap<String, Camera3D>,
    pub uis: HashMap<String, UI>,
    pub shaders: HashMap<String, Shader>,
    pub shadow_shader: Option<Shader>,
    pub active_camera: String,
    pub active_shader: String,
}

impl Default for NodeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeManager {
    pub fn new() -> NodeManager {
        NodeManager {
            models: HashMap::new(),
            cameras: HashMap::new(),
            uis: HashMap::new(),
            shaders: HashMap::new(),
            active_camera: String::new(),
            active_shader: String::new(),
            shadow_shader: None,
        }
    }

    pub fn add_model(&mut self, name: &str, model: Model) -> &mut Model {
        self.models.insert(name.to_string(), model);
        self.models.get_mut(name).unwrap()
    }

    pub fn get_model(&mut self, name: &str) -> &mut Model {
        self.models.get_mut(name).unwrap()
    }

    pub fn add_ui(&mut self, name: &str, ui: UI) -> &mut UI {
        self.uis.insert(name.to_string(), ui);
        self.uis.get_mut(name).unwrap()
    }

    pub fn add_shader(&mut self, name: &str, shader: Shader) -> &mut Shader {
        self.shaders.insert(name.to_string(), shader);
        if self.active_shader.is_empty() {
            self.active_shader = name.to_string();
        }
        self.shaders.get_mut(name).unwrap()
    }

    pub fn add_camera(&mut self, name: &str, camera: Camera3D) -> &mut Camera3D {
        self.cameras.insert(name.to_string(), camera);
        if self.active_camera.is_empty() {
            self.active_camera = name.to_string();
        }
        self.cameras.get_mut(name).unwrap()
    }

    pub fn get_camera(&mut self, name: &str) -> &mut Camera3D {
        self.cameras.get_mut(name).unwrap()
    }
}
