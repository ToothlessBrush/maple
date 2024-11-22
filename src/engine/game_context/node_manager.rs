use super::nodes::{
    camera::Camera3D, directional_light::DirectionalLight, empty::Empty, model::Model, ui::UI,
};
use crate::engine::renderer::shader::Shader;
use egui_gl_glfw::egui::util::id_type_map::SerializableAny;
use nalgebra_glm::{self as glm, Mat4, Vec3};
use std::any::Any;
use std::collections::HashMap;

pub struct NodeTransform {
    pub translation: Vec3,
    pub rotation: glm::Quat,
    pub scale: Vec3,
    pub matrix: Mat4,
}

impl Default for NodeTransform {
    fn default() -> Self {
        Self {
            translation: glm::vec3(0.0, 0.0, 0.0),
            rotation: glm::quat_identity(),
            scale: glm::vec3(1.0, 1.0, 1.0),
            matrix: glm::identity(),
        }
    }
}

//TODO: add children to nodes
pub trait Node: Any {
    type Transform;

    fn get_model_matrix(&self) -> glm::Mat4;
    fn get_transform(&self) -> &Self::Transform;
    fn define_ready<F>(&mut self, callback: F) -> &mut Self
    where
        F: 'static + FnMut(&mut Self);
    fn define_behavior<F>(&mut self, callback: F) -> &mut Self
    where
        F: 'static + FnMut(&mut Self, &mut super::GameContext);
    fn ready(&mut self);
    fn behavior(&mut self, context: &mut super::GameContext);
}

pub trait Drawable {
    fn draw(&mut self, shader: &mut Shader, camera: &Camera3D);
    fn draw_shadow(&mut self, shader: &mut Shader, light_space_matrix: &Mat4);
}

pub struct NodeManager {
    nodes: HashMap<String, Box<dyn Any>>,
    pub shaders: HashMap<String, Box<Shader>>,
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
            nodes: HashMap::new(),
            shaders: HashMap::new(),
            active_camera: String::new(),
            active_shader: String::new(),
            shadow_shader: None,
        }
    }

    pub fn add<T: Node + 'static>(&mut self, name: &str, node: T) -> &mut T {
        self.nodes.insert(name.to_string(), Box::new(node));

        //if it's the first camera added then set it as the active camera if type is Camera3D
        if std::any::type_name::<T>() == std::any::type_name::<Camera3D>()
            && self.active_camera.is_empty()
        {
            self.active_camera = name.to_string();
        }

        self.nodes
            .get_mut(name)
            .unwrap()
            .downcast_mut::<T>()
            .unwrap()
    }

    pub fn get<T: 'static + Node>(&self, name: &str) -> Option<&T> {
        self.nodes
            .get(name)
            .and_then(|node| node.downcast_ref::<T>())
    }

    pub fn get_mut<T: 'static + Node>(&mut self, name: &str) -> Option<&mut T> {
        self.nodes
            .get_mut(name)
            .and_then(|node| node.downcast_mut::<T>())
    }

    // get all nodes of a specific type as an iterator
    pub fn get_iter<T: 'static + Node>(&mut self) -> impl Iterator<Item = &mut T> {
        self.nodes
            .values_mut()
            .filter_map(|node| node.downcast_mut::<T>())
    }

    pub fn get_vec<T: 'static + Node>(&mut self) -> Vec<&mut T> {
        self.nodes
            .values_mut()
            .filter_map(|node| node.downcast_mut::<T>())
            .collect()
    }

    pub fn add_shader(&mut self, name: &str, shader: Shader) -> &mut Shader {
        self.shaders.insert(name.to_string(), Box::new(shader));
        if self.active_shader.is_empty() {
            self.active_shader = name.to_string();
        }
        self.shaders.get_mut(name).unwrap()
    }
}
