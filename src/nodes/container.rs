//! containers are used to store data within the scene they can store any clonable item. this
//! since you cant add fields to pre defined nodes containers can be used to store relevent data.
//!
//! think of it as a Node that wraps around a non-node
//!
//! # Example
//! ```rust
//! use maple::nodes::Container;
//! let container = Container::new(15.0);
//!
//! assert_eq!(*container.get_item(), 15.0);
//! ```

use super::Node;
use super::node_builder::{Builder, NodePrototype};
use crate::components::{EventReceiver, NodeTransform};
use crate::context::scene::Scene;

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

    /// to use in the container Builder
    ///
    /// most of the time this is overboard. use [`Container::new()`]
    pub fn builder(item: T) -> ContainerBuilder<T> {
        ContainerBuilder {
            item,
            prototype: NodePrototype::default(),
        }
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

/// builder implementation for container
///
/// Most of the time a builder is overkill for a container but is implemented for consistancy
pub struct ContainerBuilder<T> {
    item: T,
    prototype: NodePrototype,
}

impl<T: Clone + 'static> Builder for ContainerBuilder<T> {
    type Node = Container<T>;

    fn prototype(&mut self) -> &mut super::node_builder::NodePrototype {
        &mut self.prototype
    }

    fn build(&mut self) -> Self::Node {
        let proto = self.prototype().take();

        Container {
            transform: proto.transform,
            children: proto.children,
            events: proto.events,
            item: self.item.clone(),
        }
    }
}

impl<T> ContainerBuilder<T> {
    /// set the item stored in the container
    pub fn item(&mut self, item: T) -> &mut Self {
        self.item = item;
        self
    }
}

#[cfg(test)]
mod test {
    use crate::nodes::Container;

    #[test]
    fn test_container() {
        let container = Container::new(13);
        assert!(container.item == 13);
    }
}
