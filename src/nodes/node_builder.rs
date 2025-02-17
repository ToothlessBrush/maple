use std::default;
use std::primitive;

use egui_gl_glfw::egui::text_selection::text_cursor_state::ccursor_next_word;
use egui_gl_glfw::glfw;
use model::Primitive;
use nalgebra_glm as glm;

use crate::components::NodeTransform;
use crate::context::node_manager::Node;
use crate::context::node_manager::NodeManager;

use crate::nodes::*;

pub struct NodeBuilder<T>
where
    T: Node + Clone,
{
    pub node: T,
    pub children: NodeManager,
    pub transform: NodeTransform,
}

impl<T> NodeBuilder<T>
where
    T: Node + Clone,
{
    pub fn new(node: T) -> Self {
        NodeBuilder {
            node,
            children: NodeManager::default(),
            transform: NodeTransform::default(),
        }
    }

    pub fn camera_3d(fov: f32, aspect_ratio: f32, near: f32, far: f32) -> NodeBuilder<Camera3D> {
        NodeBuilder::new(Camera3D::new(fov, aspect_ratio, near, far))
    }

    pub fn container<U>(data: U) -> NodeBuilder<Container<U>>
    where
        U: Clone + 'static,
    {
        NodeBuilder::new(Container::new(data))
    }

    pub fn directional_light(
        shadow_distance: f32,
        shadow_resolution: u32,
    ) -> NodeBuilder<DirectionalLight> {
        NodeBuilder::new(DirectionalLight::new(shadow_distance, shadow_resolution))
    }

    pub fn empty() -> NodeBuilder<Empty> {
        NodeBuilder::new(Empty::new())
    }

    pub fn model_primitive(primitive: Primitive) -> NodeBuilder<Model> {
        NodeBuilder::new(Model::new_primitive(primitive))
    }

    pub fn model_gltf(file_path: &str) -> NodeBuilder<Model> {
        NodeBuilder::new(Model::new_gltf(file_path))
    }

    pub fn point_light(
        near_plane: f32,
        far_plane: f32,
        shadow_resolution: u32,
    ) -> NodeBuilder<PointLight> {
        NodeBuilder::new(PointLight::new(near_plane, far_plane, shadow_resolution))
    }

    pub fn ui(window: &mut glfw::PWindow) -> NodeBuilder<UI> {
        NodeBuilder::new(UI::init(window))
    }

    pub fn with_transform(&mut self, transform: NodeTransform) -> &mut Self {
        self.transform = transform;
        self
    }

    pub fn with_position(&mut self, position: glm::Vec3) -> &mut Self {
        self.transform.set_position(position);
        self
    }

    pub fn with_rotation(&mut self, rotation: glm::Quat) -> &mut Self {
        self.transform.set_rotation(rotation);
        self
    }

    pub fn with_scale(&mut self, scale: glm::Vec3) -> &mut Self {
        self.transform.set_scale(scale);
        self
    }

    pub fn add_child<U: Node>(&mut self, name: &str, node: U) -> &mut Self {
        self.children.add(name, node);
        self
    }

    pub fn build(&mut self) -> T
    where
        T: Node + Clone,
    {
        *self.node.get_children() = self.children.clone();
        //println!("{:?}", self.node.get_transform());
        *self.node.get_transform() = self.transform;
        //println!("{:?}", self.node.get_transform());
        self.node.clone()
        //println!("{:?}", clone.get_transform());
    }
}
