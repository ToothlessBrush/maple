//! Nodes are the building blocks of the scene tree. They are the objects that make up the scene.
//!
//! ## Usage
//! you can any node that implement the Node trait to the scene tree. even nodes that you create.

// re-export nodes
pub use camera::{Camera3D, Camera3DBuilder};
pub use container::{Container, ContainerBuilder};
pub use directional_light::{DirectionalLight, DirectionalLightBuilder};
pub use empty::{Empty, EmptyBuilder};
pub use model::{Model, ModelBuilder};
pub use node_builder::{Buildable, Builder};
pub use point_light::{PointLight, PointLightBuilder};
pub use ui::UI;

pub use node::Node;

pub mod node;

pub mod camera;
pub mod directional_light;
pub mod empty;
pub mod model;
pub mod node_builder;
pub mod point_light;
pub mod ui;

pub mod container;
