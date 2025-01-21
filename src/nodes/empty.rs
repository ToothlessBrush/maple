//! Empty is a node with no special functionality. it is the default node.
//!
//! ## Example
//! ```rust
//! use quaturn::game_context::nodes::empty::Empty;
//! use quaturn::game_context::GameContext;
//! use quaturn::Engine;
//! use nalgebra_glm as glm;
//!
//! let mut engine = Engine::init("example", 800, 600);
//!
//! engine.context.nodes.add("empty", Empty::new());
//!
//! //engine.begin();
//! ```

use crate::components::NodeTransform;

use crate::context::node_manager::{Behavior, Node, NodeManager, Ready};
use crate::context::GameContext;

use crate::context::node_manager::{BehaviorCallback, ReadyCallback};
use std::sync::{Arc, Mutex};

use super::{NodeBuilder, UseBehaviorCallback, UseReadyCallback};

/// Empty nodes are nodes with no special functionality.
#[derive(Clone)]
pub struct Empty {
    /// The transform of the node.
    pub transform: NodeTransform,
    /// The children of the node.
    pub children: NodeManager,

    /// the ready callback
    pub ready_callback: ReadyCallback<Empty>,
    /// the behavior callback
    pub behavior_callback: BehaviorCallback<Empty, GameContext>,
}

impl Ready for Empty {
    fn ready(&mut self) {
        if let Some(callback) = self.ready_callback.take() {
            let mut guard = callback.lock().unwrap();
            guard(self);
            drop(guard);
            self.ready_callback = Some(callback)
        }
    }
}

impl Behavior for Empty {
    fn behavior(&mut self, context: &mut GameContext) {
        // take callback out of self so we can use self later
        if let Some(callback) = self.behavior_callback.take() {
            let mut guard = callback.lock().unwrap();
            guard(self, context); //"call back"
            drop(guard); // delete stupid fucking guard because its stupid and dumb
            self.behavior_callback = Some(callback);
        }
    }
}

impl Node for Empty {
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }

    fn get_children(&mut self) -> &mut NodeManager {
        &mut self.children
    }

    fn as_ready(&mut self) -> Option<&mut (dyn Ready + 'static)> {
        Some(self)
    }

    fn as_behavior(&mut self) -> Option<&mut (dyn Behavior + 'static)> {
        Some(self)
    }
}

impl Default for Empty {
    fn default() -> Self {
        Self::new()
    }
}

impl Empty {
    ///creates a new empty node
    ///
    /// # Returns
    /// The new empty node.
    pub fn new() -> Self {
        Empty {
            transform: NodeTransform::default(),
            children: NodeManager::new(),

            ready_callback: None,
            behavior_callback: None,
        }
    }

    /// define the ready callback for the node
    ///
    /// # Arguments
    /// - `ready_function` - The function to be called when the node is ready.
    pub fn define_ready<F>(&mut self, ready_function: F) -> &mut Self
    where
        F: 'static + FnMut(&mut Self) + Sync + Send,
    {
        self.ready_callback = Some(Arc::new(Mutex::new(ready_function)));
        self
    }

    /// define the behavior callback for the node
    ///
    /// # Arguments
    /// - `behavior_function` - The function to be called when the node is behaving.
    pub fn define_behavior<F>(&mut self, behavior_function: F) -> &mut Self
    where
        F: 'static + FnMut(&mut Self, &mut GameContext) + Sync + Send,
    {
        self.behavior_callback = Some(Arc::new(Mutex::new(behavior_function)));
        self
    }
}

impl UseReadyCallback for NodeBuilder<Empty> {
    type Node = Empty;

    fn with_ready<F>(&mut self, ready_functin: F) -> &mut Self
    where
        F: 'static + FnMut(&mut Empty) + Send + Sync,
    {
        self.node.define_ready(ready_functin);
        self
    }
}

impl UseBehaviorCallback for NodeBuilder<Empty> {
    type Node = Empty;

    fn with_behavior<F>(&mut self, behavior_function: F) -> &mut Self
    where
        F: 'static + FnMut(&mut Empty, &mut GameContext) + Send + Sync,
    {
        self.node.define_behavior(behavior_function);
        self
    }
}

