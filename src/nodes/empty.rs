//! Empty is a node with no special functionality. it is the default node.
//!
use crate::components::{EventReceiver, NodeTransform};

use super::Node;
use crate::context::scene::Scene;

use super::NodeBuilder;

/// Empty nodes are nodes with no special functionality.
#[derive(Clone)]
pub struct Empty {
    /// The transform of the node.
    pub transform: NodeTransform,
    /// The children of the node.
    pub children: Scene,
    /// event handler for empty
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

/// [Empty] specific methods for [NodeBuilder]
pub trait EmptyBuilder {
    /// create a [NodeBuilder] for an [Empty] Node
    fn create() -> NodeBuilder<Empty> {
        NodeBuilder::new(Empty::new())
    }
}

impl EmptyBuilder for NodeBuilder<Empty> {}
