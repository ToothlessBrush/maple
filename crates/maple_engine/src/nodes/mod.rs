//! Nodes are the building blocks of the scene tree. They are the objects that make up the scene.
//!
//! ## Usage
//! you can any node that implement the Node trait to the scene tree. even nodes that you create.

// re-export nodes
pub use container::{Container, ContainerBuilder};
pub use empty::{Empty, EmptyBuilder};
pub use node_builder::{Buildable, Builder};

pub use node::Node;

pub mod node;

pub mod empty;
pub mod node_builder;

pub mod container;
