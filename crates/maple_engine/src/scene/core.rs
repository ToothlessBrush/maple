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
//! use maple::{
//!     context::scene::Scene,
//!     math,
//!     nodes::{Buildable, Builder, Empty},
//! };
//!
//! let mut scene = Scene::default();
//!
//! // add a node
//! scene.add(
//!     "example",
//!     Empty::builder()
//!         .position(math::vec3(10.0, 0.0, 10.0))
//!         .build(),
//! );
//!
//! // iterate over nodes
//! for (name, _node) in &scene {
//!     println!("{}", name);
//! }
//!
//! // get the node
//! let _example = scene.get::<Empty>("example");
//!
//! // remove the node
//! scene.remove("example");
//! ```

use crate::components::EventLabel;
use crate::components::node_transform::WorldTransform;
use crate::context::GameContext;
use crate::nodes::{Instanceable, Node};
use std::collections::HashMap;

/// The Scene struct is used to manage all the nodes in the scene tree.
pub struct Scene {
    /// A hashmap of all the nodes in the scene tree.
    nodes: HashMap<String, Box<dyn Node>>,
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
        }
    }

    /// adds a node to the scene tree with the given name.
    ///
    /// # Arguments
    /// - `name` - the name of the node.
    /// - `item` - either a Node or a Builder that converts into a Node.
    ///
    /// # Returns
    /// a mutable reference to the node.
    ///
    /// # Panics
    /// if the node cannot be downcast to the given type.
    ///
    /// # Note
    /// If a node with the given name already exists, a number will be appended to the name
    /// to make it unique (e.g., "player" becomes "player1", "player2", etc.).
    /// A warning will be printed when this occurs.
    ///
    /// # Examples
    /// ```rust,ignore
    /// // Both of these work:
    /// scene.add("cube1", Mesh3D::cube()); // Builder - auto-converts to Mesh3D
    /// scene.add("cube2", my_mesh); // Direct Node
    /// ```
    pub fn add<T>(&mut self, name: &str, node: T) -> &mut T
    where
        T: Node + 'static,
    {
        // Check for reserved character
        if name.contains('/') {
            panic!("'/' is a reserved character in node names");
        }

        // Find a unique name if duplicate exists
        let mut final_name = name.to_string();
        if self.nodes.contains_key(&final_name) {
            let mut counter = 1;
            loop {
                let candidate = format!("{}{}", name, counter);
                if !self.nodes.contains_key(&candidate) {
                    log::warn!(
                        "Node '{}' already exists, renaming to '{}'",
                        name, candidate
                    );
                    final_name = candidate;
                    break;
                }
                counter += 1;
            }
        }

        // Insert node
        self.nodes.insert(final_name.clone(), Box::new(node));

        // Downcast and return
        self.nodes
            .get_mut(&final_name)
            .and_then(|node| node.as_any_mut().downcast_mut::<T>())
            .expect("Failed to downcast the node")
    }

    /// remove a node from the Scene
    pub fn remove(&mut self, name: &str) -> Option<Box<dyn Node>> {
        self.nodes.remove(name)
    }

    /// this loads a scene into another by combining them
    ///
    /// Note: this will overide existing keys if there are duplicates
    pub fn merge<T>(&mut self, scene: T)
    where
        T: Into<Scene>,
    {
        let mut scene = scene.into();

        for (key, node) in scene.nodes.drain() {
            // Check if a node with the same key already exists in self.nodes
            // If it exists, replace it with the new node (overriding the previous one)
            self.nodes.insert(key, node);
        }
    }

    /// remove a bunch of nodes from an iterator of keys
    ///
    /// Note: this removes duplicate keys
    pub fn subtract<'a, I>(&mut self, keys: I)
    where
        I: IntoIterator<Item = &'a str>,
    {
        for key in keys {
            self.nodes.remove(key);
        }
    }

    /// emits an event to the scenes nodes this will trigger the event for this scenes nodes and
    /// the nodes children
    pub fn emit<E: EventLabel>(&mut self, event: &E, ctx: &mut GameContext) {
        for node in &mut self.nodes.values_mut() {
            node.trigger_event(event, ctx, WorldTransform::default());
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
                log::warn!(
                    "Could not find node by name: \"{}\" in: \"{}\"",
                    path_name, name
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
                log::warn!(
                    "Could not find node by name: \"{}\" in: \"{}\"",
                    path_name, name
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
    /// ```rust,ignore
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
                log::warn!(
                    "Could not find node by name: \"{}\" in: \"{}\"",
                    path_name, name
                );
                return None;
            }
        }

        if let Some(casted_node) = current_node.as_any().downcast_ref::<T>() {
            Some(casted_node)
        } else {
            // Warning if the node is found but the type is incorrect
            log::warn!(
                "Node found, but type mismatch for node: \"{}\". Perchance the type is wrong",
                name
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
    /// ```rust,ignore
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
                log::warn!(
                    "Could not find node by name: \"{}\" in: \"{}\"",
                    path_name, name
                );

                return None;
            }
        }

        if let Some(casted_node) = current_node.as_any_mut().downcast_mut::<T>() {
            Some(casted_node)
        } else {
            // Warning if the node is found but the type is incorrect
            log::warn!(
                "Node found, but type mismatch for node: \"{}\". Perchance the type is wrong",
                name
            );
            None
        }
    }

    /// run a callback on all nodes of a type in the entire scene tree
    pub fn for_each<T: Node + 'static>(&mut self, f: &mut impl FnMut(&mut T)) {
        let keys: Vec<String> = self.nodes.keys().cloned().collect();

        for key in keys {
            if let Some(node) = self.nodes.get_mut(&key) {
                if let Some(typed_node) = node.downcast_mut::<T>() {
                    f(typed_node);
                }

                // do the same for the children in dps
                node.get_children_mut().for_each(f);
            }
        }
    }

    /// Run a callback on all nodes of a type (immutable version)
    pub fn for_each_ref<T: Node + 'static>(&self, f: &mut impl FnMut(&T)) {
        for node in self.nodes.values() {
            if let Some(typed_node) = node.downcast::<T>() {
                f(typed_node);
            }
            node.get_children().for_each_ref(f);
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
}

/// InstancedScene is a scene that contains only instanceable nodes.
///
/// This allows for efficient cloning/instancing of entire scene hierarchies
/// where all nodes share the same underlying GPU resources but have independent
/// transforms and descriptors.
///
/// # Example
/// ```rust,ignore
/// let original_scene = InstancedScene::new();
/// // Add instanceable nodes...
///
/// // Create an instance with shared GPU resources
/// let instance1 = original_scene.instance();
/// let instance2 = original_scene.instance();
/// ```
pub struct InstancedScene {
    /// A hashmap of all instanceable nodes in the scene.
    nodes: HashMap<String, Box<dyn Instanceable>>,
}

impl Default for InstancedScene {
    fn default() -> Self {
        Self::new()
    }
}

impl InstancedScene {
    /// Creates a new empty InstancedScene.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    /// Adds an instanceable node to the scene.
    ///
    /// # Arguments
    /// - `name` - the name of the node
    /// - `node` - the instanceable node to add
    ///
    /// # Returns
    /// a mutable reference to the added node
    pub fn add<T>(&mut self, name: &str, node: T) -> &mut T
    where
        T: Instanceable + 'static,
    {
        // Check for reserved character
        if name.contains('/') {
            panic!("'/' is a reserved character in node names");
        }

        // Find a unique name if duplicate exists
        let mut final_name = name.to_string();
        if self.nodes.contains_key(&final_name) {
            let mut counter = 1;
            loop {
                let candidate = format!("{}{}", name, counter);
                if !self.nodes.contains_key(&candidate) {
                    log::warn!(
                        "Node '{}' already exists, renaming to '{}'",
                        name, candidate
                    );
                    final_name = candidate;
                    break;
                }
                counter += 1;
            }
        }

        // Insert node
        self.nodes.insert(final_name.clone(), Box::new(node));

        // Downcast and return
        self.nodes
            .get_mut(&final_name)
            .and_then(|node| node.as_any_mut().downcast_mut::<T>())
            .expect("Failed to downcast the node")
    }

    /// Creates an instance of this scene.
    ///
    /// All nodes are instanced, sharing their underlying data (buffers, materials, etc.)
    /// but with independent transforms and descriptors.
    pub fn instance(&self) -> Self {
        let mut instanced = InstancedScene::new();

        for (name, node) in &self.nodes {
            // Instance each node using its Instanceable implementation
            let node_instance = node.instance_boxed();
            instanced.nodes.insert(name.clone(), node_instance);
        }

        instanced
    }

    /// Get a reference to the nodes hashmap.
    pub fn nodes(&self) -> &HashMap<String, Box<dyn Instanceable>> {
        &self.nodes
    }

    /// Get a mutable reference to the nodes hashmap.
    pub fn nodes_mut(&mut self) -> &mut HashMap<String, Box<dyn Instanceable>> {
        &mut self.nodes
    }
}

impl Into<Scene> for InstancedScene {
    fn into(self) -> Scene {
        let mut scene = Scene::new();

        for (name, node) in self.nodes {
            // Instanceable extends Node, so we can upcast
            scene.nodes.insert(name, node as Box<dyn Node>);
        }

        scene
    }
}
