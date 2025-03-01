use egui_gl_glfw::glfw;
use model::Primitive;
use nalgebra_glm as glm;

use crate::components::Event;
use crate::components::EventReceiver;
use crate::components::NodeTransform;
use crate::context::scene::Node;
use crate::context::scene::Scene;

use crate::nodes::*;

//todo:
// I thought maybe it would be good to wrap a callback inside a predefined callback that way when the user defines a callback inside of nodebuilder they dont have to worry about downcasting and is added automatically by the NodeBuilder
// since the prototype EventHandler struct needs to call it with dyn node

pub struct NodeBuilder<T>
where
    T: Node + Clone,
{
    pub node: T,
    pub children: Scene,
    pub transform: NodeTransform,
    pub events: EventReceiver,
}

impl<T> NodeBuilder<T>
where
    T: Node + Clone,
{
    pub fn new(node: T) -> Self {
        NodeBuilder {
            node,
            children: Scene::default(),
            transform: NodeTransform::default(),
            events: EventReceiver::default(),
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

    pub fn with_rotation_euler_xyz(&mut self, rotation: glm::Vec3) -> &mut Self {
        self.transform.rotate_euler_xyz(rotation);
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

    pub fn on<F>(&mut self, event: Event, callback: F) -> &mut Self
    where
        F: FnMut(&mut T, &mut GameContext) + 'static,
    {
        self.events.on(event, callback);
        self
    }

    pub fn build(&mut self) -> T
    where
        T: Node + Clone,
    {
        *self.node.get_children_mut() = self.children.clone();
        //println!("{:?}", self.node.get_transform());
        *self.node.get_transform() = self.transform;
        //println!("{:?}", self.node.get_transform());
        *self.node.get_events() = self.events.clone();
        self.node.clone()
        //println!("{:?}", clone.get_transform());
    }
}
