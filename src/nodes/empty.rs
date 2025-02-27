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

use crate::components::{EventReceiver, NodeTransform};

use crate::context::scene::{Node, Scene};



/// Empty nodes are nodes with no special functionality.
#[derive(Clone)]
pub struct Empty {
    /// The transform of the node.
    pub transform: NodeTransform,
    /// The children of the node.
    pub children: Scene,

    pub events: EventReceiver,
}

impl Node for Empty {
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }

    fn get_children(&self) -> &Scene {
        &self.children
    }

    fn get_events(&mut self) -> &mut crate::components::EventReceiver {
        &mut self.events
    }

    fn get_children_mut(&mut self) -> &mut Scene {
        &mut self.children
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
            children: Scene::new(),
            events: EventReceiver::new(),
        }
    }
}
