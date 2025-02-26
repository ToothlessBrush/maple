//! The `node_manager` module defines and manages the scene tree, enabling hierarchical organization and updates for game objects.
//!
//! ## Features
//! - **Node Traits**: `Node`, `Ready`, and `Behavior` for defining custom nodes with initialization and per-frame logic.
//! - **Transformations**: `NodeTransform` struct for position, rotation, scale, and model matrix handling.
//! - **Scene Management**: `NodeManager` for managing child nodes and recursive scene updates.
//!
//! ## Usage
//! Implement the `Node` trait for custom objects, with optional `Ready` and `Behavior` traits for setup and updates. Use `NodeManager` to manage child nodes and relationships.
//!
//! ### Example
//! ```rust
//! //implement the Node trait for a custom node with Ready and Behavior traits
//! use quaturn::game_context::node_manager::{Node, NodeTransform, NodeManager, Ready, Behavior};
//! use quaturn::Engine;
//! use quaturn::game_context::GameContext;
//! struct CustomNode {
//!     transform: NodeTransform,
//!     children: NodeManager,
//!     /* more optional fields */
//! }
//! impl Node for CustomNode {
//!     fn get_transform(&mut self) -> &mut NodeTransform {
//!         &mut self.transform
//!     }
//!     fn get_children(&mut self) -> &mut NodeManager {
//!         &mut self.children
//!     }
//!
//!     // nodes that implement the Ready trait need to have a as_ready method to
//!     // cast to the dyn Ready object so the engine can dynamically dispatch the ready method
//!     fn as_ready(&mut self) -> Option<&mut (dyn Ready + 'static)> {
//!         Some(self)
//!     }
//!
//!     // nodes that implement the Behavior trait need to have a as_behavior method to
//!     // cast to the dyn Behavior object so the engine can dynamically dispatch the ready method
//!     fn as_behavior(&mut self) -> Option<&mut (dyn Behavior + 'static)> {
//!         Some(self)
//!     }
//! }
//!
//! impl Ready for CustomNode {
//!     fn ready(&mut self) {
//!         println!("Node ready!");
//!     }
//! }
//! impl Behavior for CustomNode {
//!     fn behavior(&mut self, _ctx: &mut GameContext) {
//!         println!("Node update!");
//!     }
//! }
//!
//! impl CustomNode {
//!     pub fn new() -> Self {
//!         Self {
//!             transform: NodeTransform::default(),
//!             children: NodeManager::new(),
//!        }
//!     }
//! }
//!
//!
//! // add an instance of the custom node to the engine
//!
//! let mut engine = Engine::init("Example", 800, 600);
//!
//! engine.context.nodes.add("custom", CustomNode::new());
//! ```

use crate::components::Event;
use crate::components::{EventReceiver, NodeTransform};
use crate::nodes::{Camera3D, Model};
use crate::renderer::shader::Shader;
use dyn_clone::DynClone;
use nalgebra_glm::{self as glm, Mat4};
use std::any::Any;
use std::collections::HashMap;
use std::error::Error;

use colored::*;

use std::fmt;

use std::sync::{Arc, Mutex};

use super::GameContext;

// /// The Ready trait is used to define that has behavior that is called when the node is ready.
// ///
// /// This is useful for nodes that need to perform some kind of setup before the game starts.
// ///
// /// # Example
// /// ```rust
// /// use quaturn::context::node_manager::{Node, NodeTransform, NodeManager, Ready};
// /// use std::any::Any;
// ///
// /// struct CustomNode {
// ///    transform: NodeTransform,
// ///    children: NodeManager,
// ///    /* more optional fields */
// /// }
// ///
// /// impl Node for CustomNode {
// ///     fn get_transform(&mut self) -> &mut NodeTransform {
// ///         &mut self.transform
// ///     }
// ///
// ///     fn get_children(&mut self) -> &mut NodeManager {
// ///         &mut self.children
// ///     }
// ///
// ///     // nodes that implement the Ready trait need to have a as_ready method to
// ///     // cast to the dyn Ready object so the engine can dynamically dispatch the ready method
// ///     fn as_ready(&mut self) -> Option<&mut (dyn Ready)> {
// ///         Some(self)
// ///     }
// /// }
// ///
// /// impl Ready for CustomNode {
// ///     fn ready(&mut self) {
// ///         println!("CustomNode is ready!");
// ///     }
// /// }
// ///
// /// impl CustomNode {
// ///     pub fn new() -> Self {
// ///         Self {
// ///             transform: NodeTransform::default(),
// ///             children: NodeManager::new(),
// ///        }
// ///    }
// /// }
// /// ```
// pub trait Ready: Node {
//     /// the ready method is called when the node is ready.
//     fn ready(&mut self);
// }

// pub type ReadyCallback<T> = Option<Arc<Mutex<dyn FnMut(&mut T) + Send + Sync>>>;

// /// The Behavior trait is used to define that has behavior that is called every frame.
// ///
// /// This is useful for nodes that need to perform some kind of logic every frame.
// ///
// /// # Example
// /// ```rust
// /// use quaturn::context::node_manager::{Node, NodeTransform, NodeManager, Behavior};
// /// use quaturn::context::GameContext;
// /// use std::any::Any;
// ///
// /// struct CustomNode {
// ///    transform: NodeTransform,
// ///    children: NodeManager,
// ///    /* more optional fields */
// /// }
// ///
// /// impl Node for CustomNode {
// ///     fn get_transform(&mut self) -> &mut NodeTransform {
// ///         &mut self.transform
// ///     }
// ///
// ///     fn get_children(&mut self) -> &mut NodeManager {
// ///         &mut self.children
// ///     }
// ///
// ///     // nodes that implement the Behavior trait need to have a as_behavior method to
// ///     // cast to the dyn Behavior object so the engine can dynamically dispatch the ready method
// ///     fn as_behavior(&mut self) -> Option<&mut (dyn Behavior)> {
// ///         Some(self)
// ///     }
// /// }
// ///
// /// impl Behavior for CustomNode {
// ///     fn behavior(&mut self, context: &mut GameContext) {
// ///         println!("CustomNode is ready!");
// ///     }
// /// }
// ///
// /// impl CustomNode {
// ///     pub fn new() -> Self {
// ///         Self {
// ///             transform: NodeTransform::default(),
// ///             children: NodeManager::new(),
// ///        }
// ///    }
// /// }
// /// ```
// pub trait Behavior: Node {
//     /// the behavior method is called every frame.
//     fn behavior(&mut self, context: &mut super::GameContext);
// }

// pub type BehaviorCallback<T, U> = Option<Arc<Mutex<dyn FnMut(&mut T, &mut U) + Send + Sync>>>;

// pub trait Casts: Any {
//     fn as_any(&self) -> &dyn Any;
//     fn as_any_mut(&mut self) -> &mut dyn Any;
// }

// impl<T: Any> Casts for T {
//     fn as_any(&self) -> &dyn Any {
//         self
//     }

//     fn as_any_mut(&mut self) -> &mut dyn Any {
//         self
//     }
// }

// TODO: Implement a more efficient way to cast to a specific trait

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
    /// use quaturn::game_context::node_manager::{Node, NodeTransform, NodeManager, Transformable};
    /// use quaturn::game_context::nodes::empty::Empty;
    /// use quaturn::Engine;
    /// use std::any::Any;
    ///
    /// use nalgebra_glm as glm;
    ///
    /// let mut engine = Engine::init("Example", 800, 600);
    /// engine.context.nodes.add("empty", Empty::new()).apply_transform(&mut |t| {
    ///     t.set_position(glm::vec3(1.0, 0.0, 0.0));
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

// pub trait ReadyCast {
//     fn as_ready(&mut self) -> Option<&mut dyn Ready>;
// }

// impl<T: Node> ReadyCast for T {
//     fn as_ready(&mut self) -> Option<&mut dyn Ready> {
//         None
//     }
// }

// impl<T: Node + Ready> ReadyCast for T {
//     fn as_ready(&mut self) -> Option<&mut dyn Ready> {
//         Some(self)
//     }
// }

// pub trait BehaviorCast {
//     fn as_behavior(&mut self) -> Option<&mut dyn Behavior>;
// }

// impl<T: Behavior> BehaviorCast for T {
//     fn as_behavior(&mut self) -> Option<&mut dyn Behavior> {
//         Some(self)
//     }
// }

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
    fn get_model_matrix(&mut self) -> &glm::Mat4 {
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
    fn get_children(&self) -> &NodeManager;

    fn get_children_mut(&mut self) -> &mut NodeManager;

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
    fn fmt_with_indent(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        // SAFETY: Temporarily convert &self to &mut self for accessing mutable methods
        let this = self as *const dyn Node as *mut dyn Node;

        // Access the transform immutably through unsafe re-borrow
        let transform = unsafe { &*(*this).get_transform() };
        let indent_str = "\t".repeat(indent); // Create indentation string

        writeln!(f, "{}Transform: {{{:?}}}", indent_str, transform)?;

        // Access children
        let children = unsafe { &mut *(*this).get_children_mut() };
        if !children.nodes.is_empty() {
            writeln!(f, "{}Children: [", indent_str)?;
            for (name, child) in children {
                writeln!(f, "{}\"{}\": {{", "\t".repeat(indent + 1), name)?; // Indent child name
                child.fmt_with_indent(f, indent + 2)?; // Recursively format child with increased indentation
                writeln!(f, "{}}}", "\t".repeat(indent + 1))?;
            }
            writeln!(f, "{}]", indent_str)?;
        }

        Ok(())
    }

    pub fn downcast<T>(&self) -> Option<&T>
    where
        T: Node,
    {
        self.as_any().downcast_ref::<T>()
    }

    pub fn downcast_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Node,
    {
        self.as_any_mut().downcast_mut::<T>()
    }

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
// dyn_clone::clone_trait_object!(FnMut(&mut dyn Node));

// pub trait BehaviorCast {
//     fn as_behavior(&mut self) -> Option<&mut dyn Behavior>;
// }

// impl<T: Behavior> BehaviorCast for T {
//     fn as_behavior(&mut self) -> Option<&mut dyn Behavior> {
//         Some(self)
//     }
// }

// impl<T: Node + Behavior> BehaviorCast for T {
//     fn as_behavior(&mut self) -> Option<&mut dyn Behavior> {
//         Some(self)
//     }
// }
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

/// The NodeManager struct is used to manage all the nodes in the scene tree.
#[derive(Clone)]
pub struct NodeManager {
    /// A hashmap of all the nodes in the scene tree.
    nodes: HashMap<String, Box<dyn Node>>,
    /// A hashmap of all the shaders in the scene.
    pub shaders: HashMap<String, Box<Shader>>,
    /// The shadow shader used to render depth maps.
    pub shadow_shader: Option<Shader>,
    /// The active camera in the scene.
    pub active_camera: String,
    /// The active shader in the scene.
    pub active_shader: String,
}

impl Default for NodeManager {
    /// the default constructor for NodeManager creates a new NodeManager with no nodes, shaders, or active camera.
    fn default() -> Self {
        Self::new()
    }
}

// copies the values of the NodeManager struct into an iterator
impl IntoIterator for NodeManager {
    type Item = (String, Box<dyn Node>);
    type IntoIter = std::collections::hash_map::IntoIter<String, Box<dyn Node>>;
    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

// returns an iterator over the nodes in the NodeManager readonly
impl<'a> IntoIterator for &'a NodeManager {
    type Item = (&'a String, &'a Box<dyn Node>);
    type IntoIter = std::collections::hash_map::Iter<'a, String, Box<dyn Node>>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter()
    }
}

// returns an iterator over the nodes in the NodeManager mutable
impl<'a> IntoIterator for &'a mut NodeManager {
    type Item = (&'a String, &'a mut Box<dyn Node>);
    type IntoIter = std::collections::hash_map::IterMut<'a, String, Box<dyn Node>>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter_mut()
    }
}

impl NodeManager {
    /// constructs a new NodeManager with no nodes, shaders, or active camera.
    pub fn new() -> NodeManager {
        NodeManager {
            nodes: HashMap::new(),
            shaders: HashMap::new(),
            active_camera: String::new(),
            active_shader: String::new(),
            shadow_shader: None,
        }
    }

    /// adds a node to the scene tree with the given name.
    ///
    /// # Arguments
    /// - `name` - the name of the node.
    /// - `node` - the node to add to the scene tree.
    ///
    /// # Returns
    /// a mutable reference to the node.
    ///
    /// # Panics
    /// if the node cannot be downcast to the given type.
    ///
    /// # Example
    /// ```rust
    /// use quaturn::game_context::nodes::empty::Empty;
    /// use quaturn::Engine;
    /// use std::any::Any;
    ///
    /// let mut engine = Engine::init("Example", 800, 600);
    ///
    /// engine.context.nodes.add("empty", Empty::new());
    /// ```
    pub fn add<T: Node + 'static>(
        &mut self,
        name: &str,
        node: T,
    ) -> Result<&mut T, Box<dyn Error>> {
        // Insert the node into the map
        if name.contains('/') {
            return Err("/ is a reserved character".into());
        }

        if self.nodes.contains_key(name) {
            return Err(format!("Node: {} already exists", name).into());
        }

        self.nodes.insert(name.to_string(), Box::new(node));

        // If it's the first camera added, set it as the active camera
        if std::any::type_name::<T>() == std::any::type_name::<Camera3D>()
            && self.active_camera.is_empty()
        {
            self.active_camera = name.to_string();
        }

        // Safely downcast and return the node
        Ok(self
            .nodes
            .get_mut(name)
            .and_then(|node| node.as_any_mut().downcast_mut::<T>())
            .expect("Failed to downcast the node"))
    }

    pub fn emit(&mut self, event: Event, ctx: &mut GameContext) {
        for node in &mut self.nodes.values_mut() {
            if event == Event::Ready {
                if let Some(camera) = node.downcast_mut::<Camera3D>() {
                    if ctx.active_camera_path.is_empty() {
                        let camera_ptr = camera.as_ptr();
                        ctx.set_main_camera(camera_ptr);
                    }
                }
            }

            node.trigger_event(event.clone(), ctx);
        }
    }

    /// get a node by name but return an immutable reference to the node
    ///
    /// # Arguments
    /// - `name` - the name of the node
    ///
    /// # Returns
    /// a reference to the node
    pub fn get_dyn(&self, name: &str) -> Option<&dyn Node> {
        self.nodes.get(name).map(|node| &**node) // We return an immutable reference
    }

    /// get a node by name but return a mutable reference to the node
    ///
    /// # Arguments
    /// - `name` - the name of the node
    ///
    /// # Returns
    /// a mutable reference to the node
    pub fn get_dyn_mut(&mut self, name: &str) -> Option<&mut dyn Node> {
        self.nodes.get_mut(name).map(|node| &mut **node) // We return a mutable reference
    }

    /// get all the nodes in the scene tree.
    ///
    /// # Returns
    /// a hashmap of all the nodes in the scene tree.
    pub fn get_all(&self) -> &HashMap<String, Box<dyn Node>> {
        &self.nodes
    }

    /// get all the nodes in the scene tree as a mutable reference.
    ///
    /// # Returns
    /// a mutable hashmap of all the nodes in the scene tree.
    pub fn get_all_mut(&mut self) -> &mut HashMap<String, Box<dyn Node>> {
        &mut self.nodes
    }

    /// get a mutable reference to a node by name or path.
    ///
    /// # Arguments
    /// - `name` - the name of the node or path to a node.
    ///
    /// # Returns
    /// a mutable reference to the node or None if not found.
    ///
    /// # Example
    /// ```rust
    /// context.nodes.get("node_name")
    /// // or
    /// context.nodes.get("path/to/node") // for nested nodes
    /// ```
    pub fn get<T: Node>(&self, name: &str) -> Option<&T> {
        let mut current_node = self.get_dyn(name.split('/').next()?)?;

        for path_name in name.split('/').skip(1) {
            if let Some(child) = current_node.get_children().get_dyn(path_name) {
                current_node = child;
            } else {
                // Warning if the node can't be found by name
                println!(
                    "{}",
                    format!(
                        "{}",
                        format!(
                            "Warning: Could not find node by name: \"{}\" in: \"{}\"",
                            path_name, name
                        )
                        .yellow()
                    )
                );
                return None;
            }
        }

        if let Some(casted_node) = current_node.as_any().downcast_ref::<T>() {
            Some(casted_node)
        } else {
            // Warning if the node is found but the type is incorrect
            println!(
                "{}",
                format!(
                    "{}",
                    format!(
            "Warning: Node found, but type mismatch for node: \"{}\". Perchance the type is wrong",
            name
        )
                    .yellow()
                )
            );

            None
        }
    }

    /// get a mutable reference to a node by name or path.
    ///
    /// # Arguments
    /// - `name` - the name of the node or path to a node.
    ///
    /// # Returns
    /// a mutable reference to the node or None if not found.
    ///
    /// # Example
    /// ```rust
    /// if let Some(node) = context.nodes.get_mut("node_name") {}
    /// // or
    /// if let Some(node) = context.nodes.get_mut("path/to/node") {} // for nested nodes
    /// ```
    pub fn get_mut<T: Node>(&mut self, name: &str) -> Option<&mut T> {
        let mut current_node = self.get_dyn_mut(name.split('/').next()?)?;

        for path_name in name.split('/').skip(1) {
            if let Some(child) = current_node.get_children_mut().get_dyn_mut(path_name) {
                current_node = child;
            } else {
                // Warning if the node can't be found by name
                use colored::*;

                println!(
                    "{}",
                    format!(
                        "Warning: Could not find node by name: \"{}\" in: \"{}\"",
                        path_name, name
                    )
                    .yellow()
                );

                return None;
            }
        }

        if let Some(casted_node) = current_node.as_any_mut().downcast_mut::<T>() {
            Some(casted_node)
        } else {
            // Warning if the node is found but the type is incorrect
            println!(
                "{}",
                format!(
                    "{}",
                    format!(
            "Warning: Node found, but type mismatch for node: \"{}\". Perchance the type is wrong",
            name
        )
                    .yellow()
                )
            );
            None
        }
    }

    /// get all nodes of a specific type as an iterator
    ///
    /// # Returns
    /// an iterator of mutable references to all nodes of the given type.
    pub fn get_iter<T: Node>(&mut self) -> impl Iterator<Item = &mut T> {
        self.nodes
            .values_mut()
            .filter_map(|node| node.as_any_mut().downcast_mut::<T>())
    }

    /// get all nodes of a specific type as a vector
    ///
    /// # Returns
    /// a vector of mutable references to all nodes of the given type.
    pub fn get_vec<T: Node>(&mut self) -> Vec<&mut T> {
        self.nodes
            .values_mut()
            .filter_map(|node| node.as_any_mut().downcast_mut::<T>())
            .collect()
    }

    /// add a shader to the scene.
    ///
    /// # Arguments
    /// - `name` - the name of the shader.
    /// - `shader` - the shader to add to the scene.
    ///
    /// # Returns
    /// a mutable reference to the shader.
    ///
    /// # Example
    /// ```rust
    /// /// use quaturn::game_context::nodes::empty::Empty;
    /// use quaturn::renderer::shader::Shader;
    /// use quaturn::Engine;
    /// use std::any::Any;
    ///
    /// let mut engine = Engine::init("Example", 800, 600);
    ///
    /// engine.context.nodes.add_shader("default", Shader::default());
    /// ```
    pub fn add_shader(&mut self, name: &str, shader: Shader) -> &mut Shader {
        self.shaders.insert(name.to_string(), Box::new(shader));
        if self.active_shader.is_empty() {
            self.active_shader = name.to_string();
        }
        self.shaders.get_mut(name).unwrap()
    }

    pub fn get_shader(&self, name: &str) -> Option<&Shader> {
        self.shaders.get(name).map(|b| b.as_ref())
    }

    pub fn get_shader_mut(&mut self, name: &str) -> Option<&mut Shader> {
        self.shaders.get_mut(name).map(|b| b.as_mut())
    }
}

// impl<T> From<&'static mut T> for *mut T
// where
//     T: Node,
// {
//     fn from(item: &'static mut T) -> Self {
//         item as *mut T
//     }
// }

// #[cfg(test)]
// mod test {

//     #[test]
//     fn impl_behavior_test() {
//         // build node
//         #[derive(Clone)]
//         struct Node {
//             transform: super::NodeTransform,
//             children: super::NodeManager,
//             events: super::EventReceiver,
//         }

//         impl super::Node for Node {
//             fn get_transform(&mut self) -> &mut super::NodeTransform {
//                 &mut self.transform
//             }

//             fn get_children(&self) -> &super::NodeManager {
//                 &self.children
//             }

//             fn get_children_mut(&mut self) -> &mut super::NodeManager {
//                 &mut self.children
//             }

//             fn get_events(&mut self) -> &mut crate::components::EventReceiver {
//                 &mut self.events
//             }

//         }

//         impl Node {
//             pub fn new() -> Self {
//                 Self {
//                     transform: super::NodeTransform::default(),
//                     children: super::NodeManager::new(),
//                     events: super::EventReceiver::new(),
//                 }
//             }
//         }

//         let mut node = Node::new();
//         let dyn_node = &mut node as &mut dyn super::Node;

//         assert_eq!(dyn_node.as_behavior().is_some(), true);
//     }

//     #[test]
//     fn impl_no_behavior_test() {
//         // build node with no behavior
//         #[derive(Clone)]
//         struct Node {
//             transform: super::NodeTransform,
//             children: super::NodeManager,
//             events: super::EventReceiver,
//         }

//         impl super::Node for Node {
//             fn get_transform(&mut self) -> &mut super::NodeTransform {
//                 &mut self.transform
//             }

//             fn get_children(&self) -> &super::NodeManager {
//                 &self.children
//             }

//             fn get_children_mut(&mut self) -> &mut super::NodeManager {
//                 &mut self.children
//             }

//             fn get_events(&mut self) -> &mut crate::components::EventReceiver {
//                 &mut self.events
//             }
//         }

//         impl Node {
//             pub fn new() -> Self {
//                 Self {
//                     transform: super::NodeTransform::default(),
//                     children: super::NodeManager::new(),
//                     events: super::EventReceiver::new(),
//                 }
//             }
//         }

//         let mut node_no_behavior = Node::new();
//         let node_dyn = &mut node_no_behavior as &mut dyn super::Node;
//         assert_eq!(node_dyn.as_behavior().is_none(), true);
//     }
// }
