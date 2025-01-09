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

use crate::context::node_manager::{Behavior, Node, NodeManager, NodeTransform, Ready};
use crate::context::GameContext;

/// Empty nodes are nodes with no special functionality.
pub struct Empty {
    /// The transform of the node.
    pub transform: NodeTransform,
    /// The children of the node.
    pub children: NodeManager,
    /// The callback to be called when the node is ready.
    ready_callback: Option<Box<dyn FnMut(&mut Self)>>,
    /// The callback to be called when the node is behaving.
    behavior_callback: Option<Box<dyn FnMut(&mut Self, &mut GameContext)>>,
}

impl Ready for Empty {
    fn ready(&mut self) {
        if let Some(mut callback) = self.ready_callback.take() {
            callback(self);
            self.ready_callback = Some(callback);
        }
    }
}

impl Behavior for Empty {
    fn behavior(&mut self, context: &mut GameContext) {
        if let Some(mut callback) = self.behavior_callback.take() {
            callback(self, context);
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
        F: 'static + FnMut(&mut Self),
    {
        self.ready_callback = Some(Box::new(ready_function));
        self
    }

    /// define the behavior callback for the node
    ///
    /// # Arguments
    /// - `behavior_function` - The function to be called when the node is behaving.
    pub fn define_behavior<F>(&mut self, behavior_function: F) -> &mut Self
    where
        F: 'static + FnMut(&mut Self, &mut GameContext),
    {
        self.behavior_callback = Some(Box::new(behavior_function));
        self
    }
}
