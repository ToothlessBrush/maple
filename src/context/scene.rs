//! The `Scene` module defines and manages the scene tree, enabling hierarchical organization and updates for game objects.
//!
//! in the engine, the context contains a root scene and every node has a child scene for managing
//! the simply handles creating storing and modifying nodes.
//!
//! Scenes can also be:
//! - `merged` - you can load a different scene into another combining them
//! - `removed` - removing a scene removes the keys from one scene in the other
//!
//! # Example
//! ```rust
//!
//!
//! let Scene = Scene::default();
//!
//! scene.add("example", NodeBuilder::<Empty>::create().build());
//! ```

use crate::components::Event;
use crate::components::node_transform::WorldTransform;
use crate::nodes::Camera3D;
use crate::nodes::Node;
use crate::renderer::shader::Shader;
use std::collections::HashMap;
use std::error::Error;

use colored::*;

use super::GameContext;

/// The Scene struct is used to manage all the nodes in the scene tree.
#[derive(Clone)]
pub struct Scene {
    /// A hashmap of all the nodes in the scene tree.
    nodes: HashMap<String, Box<dyn Node>>,
    /// A hashmap of all the shaders in the scene.
    pub shaders: HashMap<String, Box<Shader>>,
    /// The shadow shader used to render depth maps.
    pub shadow_shader: Option<Shader>,
    /// The active shader in the scene.
    pub active_shader: String,
}

impl Default for Scene {
    /// the default constructor for Scene creates a new Scene with no nodes, shaders, or active camera.
    fn default() -> Self {
        Self::new()
    }
}

// copies the values of the Scene struct into an iterator
impl IntoIterator for Scene {
    type Item = (String, Box<dyn Node>);
    type IntoIter = std::collections::hash_map::IntoIter<String, Box<dyn Node>>;
    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

// returns an iterator over the nodes in the Scene readonly
impl<'a> IntoIterator for &'a Scene {
    type Item = (&'a String, &'a Box<dyn Node>);
    type IntoIter = std::collections::hash_map::Iter<'a, String, Box<dyn Node>>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter()
    }
}

// returns an iterator over the nodes in the Scene mutable
impl<'a> IntoIterator for &'a mut Scene {
    type Item = (&'a String, &'a mut Box<dyn Node>);
    type IntoIter = std::collections::hash_map::IterMut<'a, String, Box<dyn Node>>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter_mut()
    }
}

impl Scene {
    /// constructs a new Scene with no nodes, shaders, or active camera.
    pub fn new() -> Scene {
        Scene {
            nodes: HashMap::new(),
            shaders: HashMap::new(),
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

        // Safely downcast and return the node
        Ok(self
            .nodes
            .get_mut(name)
            .and_then(|node| node.as_any_mut().downcast_mut::<T>())
            .expect("Failed to downcast the node"))
    }

    /// this loads a scene into another by combining them
    ///
    /// Note: this will overide existing keys if there are duplicates
    pub fn load(&mut self, scene: Scene) {
        for (key, node) in scene.nodes.iter() {
            // Check if a node with the same key already exists in self.nodes
            // If it exists, replace it with the new node (overriding the previous one)
            self.nodes.insert(key.clone(), node.clone());
        }
    }

    /// remove the nodes in a scene by passing another scene as a parameter
    ///
    /// Note: this removes duplicate keys
    pub fn unload(&mut self, scene: &Scene) {
        for key in scene.nodes.keys() {
            self.nodes.remove(key);
        }
    }

    /// emits an event to the scenes nodes this will trigger the event for this scenes nodes and
    /// the nodes children
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

            node.trigger_event(event.clone(), ctx, WorldTransform::default());
        }
    }

    /// get a node by name but return an immutable reference to the node
    ///
    /// # Arguments
    /// - `name` - the name of the node
    ///
    /// # Returns
    /// a reference to the node
    pub fn get_dyn_direct(&self, name: &str) -> Option<&dyn Node> {
        self.nodes.get(name).map(|node| &**node) // We return an immutable reference
    }

    /// get a node by name but return a mutable reference to the node
    ///
    /// # Arguments
    /// - `name` - the name of the node
    ///
    /// # Returns
    /// a mutable reference to the node
    pub fn get_dyn_direct_mut(&mut self, name: &str) -> Option<&mut dyn Node> {
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

    /// gets a node without a specific type
    pub fn get_dyn(&self, name: &str) -> Option<&dyn Node> {
        let mut current_node = self.get_dyn_direct(name.split('/').next()?)?;

        for path_name in name.split('/').skip(1) {
            if let Some(child) = current_node.get_children().get_dyn_direct(path_name) {
                current_node = child;
            } else {
                // Warning if the node can't be found by name
                eprintln!(
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

        Some(current_node)
    }

    /// get a node without a specific type mutably
    pub fn get_dyn_mut(&mut self, name: &str) -> Option<&mut dyn Node> {
        let mut current_node = self.get_dyn_direct_mut(name.split('/').next()?)?;

        for path_name in name.split('/').skip(1) {
            if let Some(child) = current_node
                .get_children_mut()
                .get_dyn_direct_mut(path_name)
            {
                current_node = child;
            } else {
                // Warning if the node can't be found by name
                eprintln!(
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

        Some(current_node)
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
        let mut current_node = self.get_dyn_direct(name.split('/').next()?)?;

        for path_name in name.split('/').skip(1) {
            if let Some(child) = current_node.get_children().get_dyn_direct(path_name) {
                current_node = child;
            } else {
                // Warning if the node can't be found by name
                eprintln!(
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

        if let Some(casted_node) = current_node.as_any().downcast_ref::<T>() {
            Some(casted_node)
        } else {
            // Warning if the node is found but the type is incorrect
            eprintln!(
                "{}",
                format!(
                    "Warning: Node found, but type mismatch for node: \"{}\". Perchance the type is wrong",
                    name
                )
                .yellow()
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
        let mut current_node = self.get_dyn_direct_mut(name.split('/').next()?)?;

        for path_name in name.split('/').skip(1) {
            if let Some(child) = current_node
                .get_children_mut()
                .get_dyn_direct_mut(path_name)
            {
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
                    "Warning: Node found, but type mismatch for node: \"{}\". Perchance the type is wrong",
                    name
                )
                .yellow()
            );
            None
        }
    }

    ///  collects all nodes with a specific type into a vector
    ///
    ///  because this involves borrowing we can only collect nodes immutably
    pub fn collect_items<T: Node + 'static>(&self) -> Vec<&T> {
        let mut items = Vec::new();

        for (_, node) in self {
            Self::collect_from_node::<T>(node.as_ref(), &mut items);
        }

        items
    }

    fn collect_from_node<'a, T: Node + 'static>(node: &'a dyn Node, items: &mut Vec<&'a T>) {
        if let Some(target) = node.as_any().downcast_ref::<T>() {
            items.push(target);
        }

        for child in node.get_children().get_all().values() {
            let child_node: &dyn Node = child.as_ref();
            Self::collect_from_node::<T>(child_node, items);
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

    /// returns the shader for this scene
    pub fn get_shader(&self, name: &str) -> Option<&Shader> {
        self.shaders.get(name).map(|b| b.as_ref())
    }

    /// returns the shader mutably
    pub fn get_shader_mut(&mut self, name: &str) -> Option<&mut Shader> {
        self.shaders.get_mut(name).map(|b| b.as_mut())
    }
}
