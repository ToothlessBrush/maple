use crate::graphics::camera::Camera3D;
use crate::graphics::model::Model;
use crate::graphics::shader::Shader;
use crate::utils::fps_manager::FPSManager;
use crate::utils::input_manager::InputManager;
use std::collections::HashMap;

pub struct Engine {
    models: HashMap<String, Model>,    //<name, model>
    camera: HashMap<String, Camera3D>, //<name, camera>
    active_camera: String,             //name of the active camera
    shaders: HashMap<String, Shader>,  //<name, shader>
    input_manager: InputManager,
    fps_manager: FPSManager,
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            models: HashMap::new(),
            camera: HashMap::new(),
            active_camera: String::new(),
            input_manager: InputManager::new(),
            fps_manager: FPSManager::new(),
            shaders: HashMap::new(),
        }
    }

    //add a model to the engine then return a reference to the model so the user can modify it
    pub fn add_model(&mut self, name: String, model: Model) -> &Model {
        self.models.insert(name, model);
    }

    pub fn add_camera(&mut self, name: String, camera: Camera3D) -> &Camera3D {
        self.camera.insert(name, camera);
    }

    pub fn add_shader(&mut self, name: String, shader: Shader) -> &Shader {
        self.shaders.insert(name, shader);
    }

    pub fn set_active_camera(&mut self, name: String) {
        self.active_camera = name;
    }

    pub fn begin_render(&self) {
        //call the ready callback for all models/cameras
        for model in self.models.values() {
            model.ready();
        }

        for camera in self.camera.values() {
            camera.ready();
        }
    }

    fn render(&self) {}
}
