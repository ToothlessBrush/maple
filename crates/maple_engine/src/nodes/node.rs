//! the Node trait is the base trait for every node. the scene is made up of types the implement
//! this trait.
//!
//! Node are made up of some components that all Nodes need to have such as a transform, a scene to
//! store children, and an EventReceiver to handle events. you can create your own nodes that
//! implement the node trait and add them to the scene just like any other node.
//!
//! # Example
//! ```rust
//! use maple::{
//!     Node,
//!     components::NodeTransform,
//! };
//!
//! // every node needs to have a transform.
//! // children and events are managed by the Scene.
//! #[derive(Clone, Node)]
//! pub struct CustomNode {
//!
//!     /// The transform of the node.
//!     #[transform]
//!     pub transform: NodeTransform,
//!
//!     /* other fields */
//! }
//! ```

use crate::components::NodeTransform;
use glam as math;
use std::any::Any;

/// The Node trait is used to define that a type is a node in the scene graph.
/// A node is a part of the scene tree that can be transformed and have children.
/// the node_manager only stores nodes that implement the Node trait.
pub trait Node: Any + Casting + Send + Sync {
    /// gets the model matrix of the node.
    ///
    /// # Returns
    /// the model matrix of the node.
    fn get_model_matrix(&mut self) -> &math::Mat4 {
        &self.get_transform().matrix
    }

    /// gets the transform of the node.
    ///
    /// # Returns
    /// a mutable reference to the transform of the node.
    fn get_transform(&mut self) -> &mut NodeTransform;
}

// impl fmt::Debug for dyn Node {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         // Start at the root level (no indentation)
//         self.fmt_with_indent(f, 0)
//     }
// }

impl dyn Node {
    /// downcast to a concrete value or None if not successful
    pub fn downcast<T>(&self) -> Option<&T>
    where
        T: Node,
    {
        self.as_any().downcast_ref::<T>()
    }

    /// downcast to a concrete type mutably or None if not successful
    pub fn downcast_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Node,
    {
        self.as_any_mut().downcast_mut::<T>()
    }
}

/// The Instanceable trait is used to define that a node can be efficiently instanced.
///
/// Instancing creates a new node that shares the underlying data (like buffers, materials)
/// but has its own transform and descriptor sets. This is much more efficient than cloning
/// the entire node with all its GPU resources.
pub trait Instanceable: Node {
    /// Creates an instance of this node.
    ///
    /// The instance shares the underlying data but has independent transforms.
    /// Children are NOT instanced - the instance will have an empty children scene.
    fn instance(&self) -> Self
    where
        Self: Sized;

    /// Creates a boxed instance of this node (object-safe version).
    ///
    /// This method allows instancing through a trait object.
    fn instance_boxed(&self) -> Box<dyn Instanceable>;
}

/// The Casting trait is used to define that a type can be cast to Any.
pub trait Casting {
    /// cast to Any trait object.
    fn as_any(&self) -> &dyn Any;
    /// cast to mutable Any trait object.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// blanket implementation of the Casting trait for all types that implement the Node trait.
impl<T: Node> Casting for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
