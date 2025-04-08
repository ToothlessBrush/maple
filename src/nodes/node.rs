//! the Node trait is the base trait for every node. the scene is made up of types the implement
//! this trait.
//!
//! Node are made up of some components that all Nodes need to have such as a transform, a scene to
//! store children, and an EventReceiver to handle events. you can create your own nodes that
//! implement the node trait and add them to the scene just like any other node.
//!
//! # Example
//! ```rust
//! use quaturn::nodes::{Node, NodeBuilder};
//!
//! #[derive(Clone)] // Nodes need Clone trait
//! struct CustomNode {
//!     transform: NodeTransform,
//!     children: Scene,
//!     events: EventReceiver,
//!     
//!     /* optional fields */
//!     custom_field: i32,
//! }
//!
//! impl Node for CustomNode {
//!     fn get_events(&mut self) -> &mut EventReceiver {
//!         &mut self.events
//!     }
//!     fn get_children(&self) -> &Scene {
//!         &self.children
//!     }
//!     fn get_transform(&mut self) -> &mut NodeTransform {
//!         &mut self.transform
//!     }
//!     fn get_children_mut(&mut self) -> &mut Scene {
//!         &mut self.children
//!     }
//! }
//!
//! trait CustomNodeBuilder {
//!     fn set_custom_field(&mut self, item: i32) -> &mut Self;
//! }
//!
//! impl CustomNodeBuilder for NodeBuilder<CustomNode> {
//!     fn set_custom_field(&mut self, item: i32) -> &mut Self {
//!         self.node.custom_field = item;
//!         self
//!     }
//! }
//! ```

use crate::components::{Event, EventReceiver, NodeTransform};
use crate::context::scene::Scene;
use crate::context::GameContext;
use crate::nodes::Camera3D;
use crate::renderer::shader::Shader;
use nalgebra_glm as math;
use std::any::Any;
use std::fmt;

use dyn_clone::DynClone;

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

/// The Node trait is used to define that a type is a node in the scene graph.
/// A node is a part of the scene tree that can be transformed and have children.
/// the node_manager only stores nodes that implement the Node trait.
pub trait Node: Any + Casting + DynClone {
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
    pub fn trigger_event(&mut self, event: Event, ctx: &mut GameContext) {
        let mut events = std::mem::take(self.get_events());
        events.trigger(event.clone(), self, ctx);
        *self.get_events() = events;

        for (_, node) in self.get_children_mut() {
            node.trigger_event(event.clone(), ctx);
        }
    }
}

dyn_clone::clone_trait_object!(Node);

/// The Drawable trait is used to define that a type can be drawn.
pub trait Drawable {
    /// draws the object using the given shader and camera.
    ///
    /// # Arguments
    /// - `shader` - the shader to use to draw the object.
    /// - `camera` - the camera to use to draw the object.
    fn draw(
        &mut self,
        shader: &mut Shader,
        camera: (&Camera3D, NodeTransform),
        parent_transform: NodeTransform,
    );
    /// draws the object using the given shader and light space matrix for rendering a depth map from the lights perspective.
    ///
    /// # Arguments
    /// - `shader` - the shader to use to draw the object.
    /// - `light_space_matrix` - the light space matrix to use to draw the object.
    fn draw_shadow(&mut self, shader: &mut Shader, parent_transform: NodeTransform);
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use quaturn::game_context::node_manager::{Node, NodeTransform, Scene, Transformable};
    /// use quaturn::game_context::nodes::empty::Empty;
    /// use quaturn::Engine;
    /// use std::any::Any;
    ///
    /// use nalgebra_glm as math;
    ///
    /// let mut engine = Engine::init("Example", 800, 600);
    /// engine.context.nodes.add("empty", Empty::new()).apply_transform(&mut |t| {
    ///     t.set_position(math::vec3(1.0, 0.0, 0.0));
    /// });
    /// ```
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
        // if let Some(model) = self.as_any_mut().downcast_mut::<Model>() {
        //     for node in &mut model.nodes {
        //         operation(&mut node.transform);
        //     }
        // }

        // for child in self.get_children().get_all_mut().values_mut() {
        //     let child_node: &mut dyn Node = &mut **child;
        //     apply_transform(child_node, operation);
        // }
        self
    }
}

impl Transformable for dyn Node {
    fn apply_transform<F>(&mut self, operation: &mut F) -> &mut Self
    where
        F: FnMut(&mut NodeTransform),
    {
        operation(self.get_transform());
        // if let Some(model) = self.as_any_mut().downcast_mut::<Model>() {
        //     for node in &mut model.nodes {
        //         operation(&mut node.transform);
        //     }
        // }
        // for child in self.get_children().get_all_mut().values_mut() {
        //     let child_node: &mut dyn Node = &mut **child;
        //     apply_transform(child_node, operation);
        // }
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

    // if let Some(model) = node.as_any_mut().downcast_mut::<Model>() {
    //     for node in &mut model.nodes {
    //         operation(&mut node.transform);
    //     }
    // }

    // for child in node.get_children().get_all_mut().values_mut() {
    //     let child_node: &mut dyn Node = &mut **child;
    //     apply_transform(child_node, operation);
    //     //println!("processing children");
    // }
}
