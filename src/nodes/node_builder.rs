use std::default;

use crate::components::NodeTransform;
use crate::context::node_manager::Node;
use crate::context::node_manager::NodeManager;

pub struct NodeBuilder<T: Node> {
    node: T,
    children: NodeManager,
    transform: NodeTransform,
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

    pub fn add_child<U: Node + 'static>(&mut self, name: &str, node: U) -> &mut Self {
        self.children.add(name, node);
        self
    }

    pub fn build(&mut self) -> T {
        *self.node.get_children() = self.children.clone();
        *self.node.get_transform() = self.transform;
        self.node.clone()
    }
}
