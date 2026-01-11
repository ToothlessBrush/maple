//! Empty is a node with no special functionality. it is the default node.
//!
//! This module provides the Empty Node which can be used as a placeholder, group object, or
//! used to define general behavior.
//!
//! # Notes
//! While the Empty node has no special functionality it still contains a transform, children, and
//! events.
use crate::components::{EventReceiver, NodeTransform};

use super::{
    Node,
    node_builder::{Buildable, Builder, NodePrototype},
};

/// Empty nodes are nodes with no special functionality.
pub struct Empty {
    /// The transform of the node.
    pub transform: NodeTransform,
    /// event handler for empty
    pub events: EventReceiver,
}

impl Node for Empty {
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }

    fn get_events(&mut self) -> &mut crate::components::EventReceiver {
        &mut self.events
    }
}

impl super::Instanceable for Empty {
    fn instance(&self) -> Self {
        Empty {
            transform: self.transform,
            events: self.events.clone(),
        }
    }

    fn instance_boxed(&self) -> Box<dyn super::Instanceable> {
        Box::new(self.instance())
    }
}

impl Default for Empty {
    fn default() -> Self {
        Empty {
            transform: NodeTransform::default(),
            events: EventReceiver::new(),
        }
    }
}

impl Buildable for Empty {
    type Builder = EmptyBuilder;

    fn builder() -> Self::Builder {
        EmptyBuilder {
            prototype: NodePrototype::default(),
        }
    }
}

/// builder for the [`Empty`]
pub struct EmptyBuilder {
    prototype: NodePrototype,
}

impl Builder for EmptyBuilder {
    type Node = Empty;

    fn prototype(&mut self) -> &mut NodePrototype {
        &mut self.prototype
    }

    fn build(self) -> Self::Node {
        Empty {
            transform: self.prototype.transform,
            events: self.prototype.events,
        }
    }
}

// /// [Empty] specific methods for [NodeBuilder]
// pub trait EmptyBuilder {
//     /// create a [NodeBuilder] for an [Empty] Node
//     fn create() -> NodeBuilder<Empty> {
//         NodeBuilder::new(Empty::new())
//     }
// }
//
// impl EmptyBuilder for NodeBuilder<Empty> {}
