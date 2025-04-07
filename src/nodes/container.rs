//! containers are used to store data within the scene they can store any clonable item. this
//! since you cant add fields to pre defined nodes containers can be used to store relevent data.
//!
//! # Example
//! ```rust
//! let container = NodeBuilder::<Container<f32>>::create(15.0).build();
//!
//! assert!(container.get_item(), 15.0);
//! ```

use super::Node;
use crate::components::{EventReceiver, NodeTransform};
use crate::context::scene::Scene;
use crate::nodes::NodeBuilder;

/// containers can store arbitrary data with the scene
#[derive(Clone)]
pub struct Container<T> {
    item: T,
    transform: NodeTransform,
    children: Scene,
    events: EventReceiver,
}

impl<T> Container<T> {
    /// create a container with a contained item
    pub fn new(item: T) -> Container<T>
    where
        T: Clone,
    {
        Container {
            item,
            transform: NodeTransform::default(),
            children: Scene::default(),
            events: EventReceiver::default(),
        }
    }

    /// get the stored item
    pub fn get_item(&self) -> &T {
        &self.item
    }

    /// get the stored item mut
    pub fn get_item_mut(&mut self) -> &mut T {
        &mut self.item
    }
}

impl<T> Node for Container<T>
where
    T: Clone + 'static,
{
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }

    fn get_children(&self) -> &Scene {
        &self.children
    }

    fn get_children_mut(&mut self) -> &mut Scene {
        &mut self.children
    }

    fn get_events(&mut self) -> &mut crate::components::EventReceiver {
        &mut self.events
    }
}

/// [NodeBuilder] for [Container]
pub trait ContainerBuilder<T> {
    /// create a ContainerBulder for a given item
    fn create(item: T) -> NodeBuilder<Container<T>>
    where
        T: Clone + 'static,
    {
        NodeBuilder::new(Container::<T>::new(item))
    }
}

impl<T: Clone + 'static> ContainerBuilder<T> for NodeBuilder<Container<T>> {}

#[cfg(test)]
mod test {
    #[test]
    fn test_container() {
        use super::ContainerBuilder;
        let _container = super::NodeBuilder::<super::Container<f32>>::create(13.0).build();
    }
}
