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

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::components::EventLabel;
use crate::components::node_transform::WorldTransform;
use crate::context::GameContext;
use crate::nodes::node::{NodeMut, NodeRef};
use crate::nodes::{Instanceable, Node};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

/// The Scene struct is used to manage all the nodes in the scene tree.
pub struct Scene {
    /// A hashmap of all the nodes in the scene tree.
    nodes: HashMap<String, Arc<RwLock<Box<dyn Node>>>>,
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for Scene {
    type Item = (String, Arc<RwLock<Box<dyn Node>>>);
    type IntoIter = std::collections::hash_map::IntoIter<String, Arc<RwLock<Box<dyn Node>>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

impl<'a> IntoIterator for &'a Scene {
    type Item = (&'a String, &'a Arc<RwLock<Box<dyn Node>>>);
    type IntoIter = std::collections::hash_map::Iter<'a, String, Arc<RwLock<Box<dyn Node>>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter()
    }
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            nodes: HashMap::new(),
        }
    }

    pub fn add<T>(&mut self, name: &str, node: T)
    where
        T: Node + 'static,
    {
        if name.contains('/') {
            panic!("'/' is a reserved character in node names");
        }

        let mut final_name = name.to_string();
        if self.nodes.contains_key(&final_name) {
            let mut counter = 1;
            loop {
                let candidate = format!("{}{}", name, counter);
                if !self.nodes.contains_key(&candidate) {
                    log::warn!(
                        "Node '{}' already exists, renaming to '{}'",
                        name,
                        candidate
                    );
                    final_name = candidate;
                    break;
                }
                counter += 1;
            }
        }

        self.nodes
            .insert(final_name, Arc::new(RwLock::new(Box::new(node))));
    }

    pub fn remove(&mut self, name: &str) -> Option<Arc<RwLock<Box<dyn Node>>>> {
        self.nodes.remove(name)
    }

    pub fn merge<T>(&mut self, scene: T)
    where
        T: Into<Scene>,
    {
        let mut scene = scene.into();

        for (key, node) in scene.nodes.drain() {
            self.nodes.insert(key, node);
        }
    }

    pub fn subtract<'a, I>(&mut self, keys: I)
    where
        I: IntoIterator<Item = &'a str>,
    {
        for key in keys {
            self.nodes.remove(key);
        }
    }

    pub fn emit<E: EventLabel>(&self, event: &E, ctx: &GameContext) {
        for node in self.nodes.values() {
            node.write()
                .trigger_event(event, ctx, WorldTransform::default());
        }
    }

    // Get Arc to a node for sharing/cloning
    pub fn get_node_arc(&self, name: &str) -> Option<Arc<RwLock<Box<dyn Node>>>> {
        self.nodes.get(name).map(Arc::clone)
    }

    pub fn get_dyn_direct<'a>(&'a self, name: &str) -> Option<NodeRef<'a, dyn Node>> {
        self.nodes
            .get(name)
            .map(|cell| NodeRef::new(RwLockReadGuard::map(cell.read(), |boxed| &**boxed)))
    }

    pub fn get_dyn_direct_mut<'a>(&'a self, name: &str) -> Option<NodeMut<'a, dyn Node>> {
        self.nodes
            .get(name)
            .map(|cell| NodeMut::new(RwLockWriteGuard::map(cell.write(), |boxed| &mut **boxed)))
    }

    pub fn get_all(&self) -> &HashMap<String, Arc<RwLock<Box<dyn Node>>>> {
        &self.nodes
    }

    pub fn get_dyn<'a>(&'a self, name: &str) -> Option<NodeRef<'a, dyn Node>> {
        self.nodes
            .get(name)
            .map(|cell| NodeRef::new(RwLockReadGuard::map(cell.read(), |boxed| &**boxed)))
    }

    pub fn get_dyn_mut<'a>(&'a self, name: &str) -> Option<NodeMut<'a, dyn Node>> {
        self.nodes
            .get(name)
            .map(|cell| NodeMut::new(RwLockWriteGuard::map(cell.write(), |boxed| &mut **boxed)))
    }

    pub fn get<'a, T: Node>(&'a self, name: &str) -> Option<NodeRef<'a, T>> {
        let cell = self.nodes.get(name)?;
        let borrowed = cell.read();
        if borrowed.as_any().is::<T>() {
            Some(NodeRef::new(RwLockReadGuard::map(borrowed, |boxed| {
                boxed.as_any().downcast_ref::<T>().unwrap()
            })))
        } else {
            log::warn!("Node found, but type mismatch for node: \"{}\"", name);
            None
        }
    }

    pub fn get_mut<'a, T: Node>(&'a self, name: &str) -> Option<NodeMut<'a, T>> {
        let cell = self.nodes.get(name)?;
        let borrowed = cell.write();
        if borrowed.as_any().is::<T>() {
            Some(NodeMut::new(RwLockWriteGuard::map(borrowed, |boxed| {
                boxed.as_any_mut().downcast_mut::<T>().unwrap()
            })))
        } else {
            log::warn!("Node found, but type mismatch for node: \"{}\"", name);
            None
        }
    }
    pub fn for_each<T: Node + 'static>(&self, f: &mut impl FnMut(&mut T)) {
        for cell in self.nodes.value < s() {
            let mut node = cell.write();
            if let Some(typed_node) = node.as_any_mut().downcast_mut::<T>() {
                f(typed_node);
            }
            node.get_children().for_each(f);
        }
    }

    pub fn for_each_ref<T: Node + 'static>(&self, f: &mut impl FnMut(&T)) {
        for cell in self.nodes.values() {
            let node = cell.read();
            if let Some(typed_node) = node.as_any().downcast_ref::<T>() {
                f(typed_node);
            }
            node.get_children().for_each_ref(f);
        }
    }

    pub fn get_iter<'a, T: Node>(&'a self) -> impl Iterator<Item = NodeRef<'a, T>> + 'a {
        self.nodes.values().filter_map(|cell| {
            let borrowed = cell.read();
            if borrowed.as_any().is::<T>() {
                Some(NodeRef::new(RwLockReadGuard::map(borrowed, |boxed| {
                    boxed.as_any().downcast_ref::<T>().unwrap()
                })))
            } else {
                None
            }
        })
    }

    pub fn get_iter_mut<'a, T: Node>(&'a self) -> impl Iterator<Item = NodeMut<'a, T>> + 'a {
        self.nodes.values().filter_map(|cell| {
            let borrowed = cell.write();
            if borrowed.as_any().is::<T>() {
                Some(NodeMut::new(RwLockWriteGuard::map(borrowed, |boxed| {
                    boxed.as_any_mut().downcast_mut::<T>().unwrap()
                })))
            } else {
                None
            }
        })
    }

    pub fn get_vec<'a, T: Node>(&'a self) -> Vec<NodeRef<'a, T>> {
        self.get_iter::<T>().collect()
    }

    pub fn get_vec_mut<'a, T: Node>(&'a self) -> Vec<NodeMut<'a, T>> {
        self.get_iter_mut::<T>().collect()
    }
}
