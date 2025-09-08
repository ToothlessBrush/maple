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
//!     .position(math::vec3(10.0, 0.0, 0.0))
//!     .scale_factor(10.0)
//!
//!     // Add child nodes
//!     .add_child("speed", Container::new(10.0))
//!
//!     // Define custom behavior with a callback
//!     .on(Event::Update, |node, ctx| {
//!         let Some(speed) = node
//!             .children
//!             .get::<Container<f32>>("speed")
//!             .map(|c| c.get_item())
//!         else {
//!             return;
//!         };
//!
//!         node.transform.position.x += *speed * ctx.frame.time_delta_f32;
//!     })
//!
//!     .build();
//! ```

use crate::components::event_reciever::EventParam;
use crate::components::event_reciever::IntoEventFunction;
use crate::context::GameContext;
use glam as math;

use super::Node;
use crate::components::Event;
use crate::components::EventReceiver;
use crate::components::NodeTransform;
use crate::scene::Scene;

/// a prototype node contains all the components that all nodes have but nothing else
#[derive(Default)]
pub struct NodePrototype {
    /// a nodes transform
    pub transform: NodeTransform,
    /// a nodes events
    pub events: EventReceiver,
    /// a nodes children
    pub children: Scene,
}

impl NodePrototype {
    /// take ownership of the prototype
    pub fn take(&mut self) -> Self {
        Self {
            transform: std::mem::take(&mut self.transform),
            children: std::mem::take(&mut self.children),
            events: std::mem::take(&mut self.events),
        }
    }
}

/// the builder trait contains a bunch of default building methods for a builable node.
///
/// things such as a nodes transform, children, and events are exposed here for building
pub trait Builder {
    /// the type of node to build
    type Node: Node;
    /// get the prototype to modify its components
    fn prototype(&mut self) -> &mut NodePrototype;
    /// builds the node
    fn build(&mut self) -> Self::Node;

    /// sets the transform of the node
    fn transform(&mut self, transform: NodeTransform) -> &mut Self {
        self.prototype().transform = transform;
        self
    }

    /// set the position of the node
    fn position(&mut self, position: math::Vec3) -> &mut Self {
        self.prototype().transform.position = position;
        self
    }

    /// set the rotation of the node
    fn rotation(&mut self, rotation: math::Quat) -> &mut Self {
        self.prototype().transform.rotation = rotation;
        self
    }

    /// set the rotation of the node with angles in xyz order
    fn rotation_euler_xyz(&mut self, rotation: math::Vec3) -> &mut Self {
        self.prototype().transform.set_euler_xyz(rotation);
        self
    }

    /// scale the node
    fn scale(&mut self, scale: math::Vec3) -> &mut Self {
        self.prototype().transform.scale = scale;
        self
    }

    /// scale all axis of node with a single factor
    fn scale_factor(&mut self, scale_factor: f32) -> &mut Self {
        self.prototype().transform.scale *= scale_factor;
        self
    }

    /// adds event behavior to a node such as on ready or update
    ///
    /// # Example
    ///  ```rust
    ///  use maple::components::Event;
    ///  use maple::nodes::{Empty, Buildable, Builder};
    ///  use maple::math;
    ///
    ///  Empty::builder()
    ///      .on(Event::Update, move |node, context| {
    ///         // called on every frame
    ///         node.transform.position += math::vec3(1.0, 0.0 ,0.0);
    ///      })
    ///      .build();
    ///  ```
    fn on<F, Params>(&mut self, event: Event, callback: F) -> &mut Self
    where
        F: IntoEventFunction<Params> + 'static,
    {
        self.prototype().events.on(event, callback);
        self
    }

    /// adds a child node
    ///
    /// child nodes transforms are relative to their parents and the update order is after the
    /// parent
    fn add_child<T: Node>(&mut self, name: &str, child: T) -> &mut Self {
        self.prototype().children.add(name, child);
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
