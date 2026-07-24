//! containers are used to store data within the scene they can store any clonable item. this
//! since you cant add fields to pre defined nodes containers can be used to store relevent data.
//!
//! think of it as a Node that wraps around a non-node
//!
//! # Example
//! ```rust
//! # use maple_engine::nodes::Container;
//! let container = Container::new(15.0);
//!
//! assert_eq!(*container, 15.0);
//! ```

use std::ops::{Deref, DerefMut};

use super::Node;
use super::node_builder::{Builder, NodePrototype};
use crate::components::NodeTransform;

/// containers can store arbitrary data with the scene
pub struct Container<T> {
    item: T,
    transform: NodeTransform,
}

impl<T> Container<T> {
    /// create a container with a contained item
    pub fn new(item: T) -> Container<T> {
        Container {
            item,
            transform: NodeTransform::default(),
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
    T: Send + Sync + 'static,
{
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }
}

impl<T: Default> Default for Container<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> From<T> for Container<T> {
    fn from(item: T) -> Self {
        Container::new(item)
    }
}

impl<T> AsRef<T> for Container<T> {
    fn as_ref(&self) -> &T {
        &self.item
    }
}

impl<T> AsMut<T> for Container<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.item
    }
}

impl<T> Clone for Container<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            item: self.item.clone(),
            transform: self.transform.clone(),
        }
    }
}

impl<T> Deref for Container<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<T> DerefMut for Container<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}

/// builder implementation for container
///
/// Most of the time a builder is overkill for a container but is implemented for consistancy
pub struct ContainerBuilder<T> {
    item: T,
    prototype: NodePrototype,
}

impl<T: Clone + Send + Sync + 'static> Builder for ContainerBuilder<T> {
    type Node = Container<T>;

    fn prototype(&mut self) -> &mut super::node_builder::NodePrototype {
        &mut self.prototype
    }

    fn build(self) -> Self::Node {
        Container {
            transform: self.prototype.transform,
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
