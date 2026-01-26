//! the NodeBuilder is used to help create and build nodes within a scene. instead of having tons
//! of parameters NodeBuilder splits node properties into different methods which decreases tedious
//! code and increases readability
//!
//! # Example
//! ```rust
//! use maple::components::Event;
//! use maple::math;
//! use maple::nodes::{Buildable, Builder, Container, Empty, Node};
//!
//! let node = Empty::builder()
//!     // Modify the node's initial transform
//!     .position((10.0, 0.0, 0.0))
//!     .scale_factor(10.0)
//!     .build();
//! ```

use glam as math;
use glam::Vec3;

use super::Node;
use crate::components::NodeTransform;

/// a prototype node contains all the components that all nodes have but nothing else
#[derive(Default)]
pub struct NodePrototype {
    /// a nodes transform
    pub transform: NodeTransform,
}

impl NodePrototype {
    /// take ownership of the prototype
    pub fn take(&mut self) -> Self {
        Self {
            transform: std::mem::take(&mut self.transform),
        }
    }
}

/// the builder trait contains a bunch of default building methods for a builable node.
///
/// things such as a nodes transform, children, and events are exposed here for building
pub trait Builder: Sized {
    /// the type of node to build
    type Node: Node;
    /// get the prototype to modify its components
    fn prototype(&mut self) -> &mut NodePrototype;
    /// builds the node
    fn build(self) -> Self::Node;

    /// sets the transform of the node
    fn transform(mut self, transform: NodeTransform) -> Self {
        self.prototype().transform = transform;
        self
    }

    /// set the position of the node
    fn position(mut self, position: impl Into<Vec3>) -> Self {
        self.prototype().transform.position = position.into();
        self
    }

    /// set the rotation of the node
    fn rotation(mut self, rotation: math::Quat) -> Self {
        self.prototype().transform.rotation = rotation;
        self
    }

    /// set the rotation of the node with angles in xyz order
    fn rotation_euler_xyz(mut self, rotation: impl Into<Vec3>) -> Self {
        self.prototype().transform.set_euler_xyz(rotation);
        self
    }

    /// scale the node
    fn scale(mut self, scale: impl Into<Vec3>) -> Self {
        self.prototype().transform.scale = scale.into();
        self
    }

    /// scale all axis of node with a single factor
    fn scale_factor(mut self, scale_factor: f32) -> Self {
        self.prototype().transform.scale *= scale_factor;
        self
    }
}

/// Buildable nodes have a builder to configure nodes before they are added into a scene
pub trait Buildable {
    /// Node Specific Builder
    type Builder: Builder<Node = Self>;

    /// returns the Builder implementation for a given node
    fn builder() -> Self::Builder;
}
