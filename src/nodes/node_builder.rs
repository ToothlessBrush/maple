use std::default;

use nalgebra_glm as glm;

use crate::components::NodeTransform;
use crate::context::node_manager::Node;
use crate::context::node_manager::NodeManager;

pub struct NodeBuilder<T: Node> {
    pub node: T,
    pub children: NodeManager,
    pub transform: NodeTransform,
}

impl<T: Node + Clone> NodeBuilder<T> {
    pub fn new(node: T) -> Self {
        NodeBuilder {
            node,
            children: NodeManager::default(),
            transform: NodeTransform::default(),
        }
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

    pub fn build(&mut self) -> T {
        *self.node.get_children() = self.children.clone();
        //println!("{:?}", self.node.get_transform());
        *self.node.get_transform() = self.transform;
        //println!("{:?}", self.node.get_transform());
        self.node.clone()
        //println!("{:?}", clone.get_transform());
    }
}
