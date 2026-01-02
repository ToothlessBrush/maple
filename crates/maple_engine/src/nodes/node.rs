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
//!     components::{NodeTransform, EventReceiver},
//!     context::Scene,
//! };
//!
//! // every node needs to be clonable and have a transform, children, and EventReceiver.
//! #[derive(Clone, Node)]
//! pub struct CustomNode {
//!
//!     /// The transform of the node.
//!     #[transform]
//!     pub transform: NodeTransform,
//!
//!     /// The children of the node.
//!     #[children]
//!     pub children: Scene,
//!
//!     /// event handler for empty
//!     #[events]
//!     pub events: EventReceiver,
//!
//!     /* other fields */
//! }
//! ```

use crate::components::EventLabel;
use crate::components::node_transform::WorldTransform;
use crate::components::{EventReceiver, NodeTransform};
use crate::context::GameContext;
use crate::scene::Scene;
use glam as math;
use std::any::Any;
use std::fmt;

/// The Node trait is used to define that a type is a node in the scene graph.
/// A node is a part of the scene tree that can be transformed and have children.
/// the node_manager only stores nodes that implement the Node trait.
pub trait Node: Any + Casting {
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

    /// gets the children of the node.
    ///
    /// # Returns
    /// a mutable reference to the children of the node.
    fn get_children(&self) -> &Scene;

    /// get the nodes children mutably
    fn get_children_mut(&mut self) -> &mut Scene;

    /// get the nodes events
    fn get_events(&mut self) -> &mut EventReceiver;
}

impl fmt::Debug for dyn Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Start at the root level (no indentation)
        self.fmt_with_indent(f, 0)
    }
}

impl dyn Node {
    // adds a tab to child nodes so its more readable
    pub(crate) fn fmt_with_indent(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        // SAFETY: Temporarily convert &self to &mut self for accessing mutable methods
        let this = self as *const dyn Node as *mut dyn Node;

        // Access the transform immutably through unsafe re-borrow
        let transform = unsafe { &*(*this).get_transform() };
        let indent_str = "\t".repeat(indent); // Create indentation string

        writeln!(f, "{}Transform: {{{:?}}}", indent_str, transform)?;

        // Access children
        let children = unsafe { &mut *(*this).get_children_mut() };
        if !children.get_all().is_empty() {
            writeln!(f, "{}Children: [", indent_str)?;
            for (name, child) in children {
                writeln!(f, "{}\"{}\": {{", "\t".repeat(indent + 1), name)?; // Indent child name
                (*child).fmt_with_indent(f, indent + 2)?; // Recursively format child with increased indentation
                writeln!(f, "{}}}", "\t".repeat(indent + 1))?;
            }
            writeln!(f, "{}]", indent_str)?;
        }

        Ok(())
    }

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

    /// trigger an event within a nodes [EventReceiver]
    ///
    /// # Arguements
    /// - `event` - the event to trigger
    /// - `ctx` - the engines context
    pub fn trigger_event<E: EventLabel>(
        &mut self,
        event: &E,
        ctx: &mut GameContext,
        parent_space: WorldTransform,
    ) {
        // update global transform before event is triggered
        self.get_transform().get_world_space(parent_space);
        let new_world_space = *self.get_transform().world_space();

        let mut events = std::mem::take(self.get_events());
        events.trigger(event, self, ctx);
        *self.get_events() = events;

        for (_, node) in self.get_children_mut() {
            node.trigger_event(event, ctx, new_world_space);
        }
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

/// The Transformable trait is used to define that a node can be transformed.
pub trait Transformable {
    /// applies a transformation to the node while still retruning itself. this way you can embed the trasnforms into method chaining.
    ///
    /// # Arguments
    /// - `operation` - the operation to apply to the node and all of its children.
    ///
    /// # Returns
    /// a mutable reference to the node.
    fn apply_transform<F>(&mut self, operation: &mut F) -> &mut Self
    where
        F: FnMut(&mut NodeTransform);
}

// implement the Transformable trait for all types that implement the Node trait
impl<T: Node> Transformable for T {
    fn apply_transform<F>(&mut self, operation: &mut F) -> &mut Self
    where
        F: FnMut(&mut NodeTransform),
    {
        operation(self.get_transform());
        self
    }
}

impl Transformable for dyn Node {
    fn apply_transform<F>(&mut self, operation: &mut F) -> &mut Self
    where
        F: FnMut(&mut NodeTransform),
    {
        operation(self.get_transform());
        self
    }
}

/// function that applies a transformation to a node and all of its children.
///
/// # Arguments
/// - `node` - the node to apply the transformation to.
/// - `operation` - the operation to apply to the node and all of its children.
pub fn apply_transform<F>(node: &mut dyn Node, operation: &mut F)
where
    F: FnMut(&mut NodeTransform),
{
    operation(node.get_transform());
}
