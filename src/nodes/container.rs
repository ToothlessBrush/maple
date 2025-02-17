use crate::components::NodeTransform;
use crate::context::node_manager::{Node, NodeManager};

#[derive(Clone)]
pub struct Container<T> {
    data: T,
    transform: NodeTransform,
    children: NodeManager,
}

impl<T> Container<T> {
    pub fn new(data: T) -> Container<T>
    where
        T: Clone,
    {
        Container {
            data,
            transform: NodeTransform::default(),
            children: NodeManager::default(),
        }
    }

    pub fn get_data(&self) -> &T {
        &self.data
    }

    pub fn get_data_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

impl<T> Node for Container<T>
where
    T: Clone + 'static,
{
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }

    fn get_children(&mut self) -> &mut NodeManager {
        &mut self.children
    }
}
