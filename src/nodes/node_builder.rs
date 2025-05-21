//! the NodeBuilder is used to help create and build nodes within a scene. instead of having tons
//! of parameters NodeBuilder splits node properties into different methods which decreases tedious
//! code and increases readability
//!
//! # Example
//! ```rust
//! use maple::nodes::{NodeBuilder, Empty, EmptyBuilder};
//! use maple::math;
//!
//! let node = NodeBuilder::<Empty>::create()
//!     .with_position(math::vec3(10.0, 0.0, 0.0))
//!     .build();
//! ```

use nalgebra_glm as math;

use super::Node;
use crate::components::Event;
use crate::components::EventReceiver;
use crate::components::NodeTransform;
use crate::context::scene::Scene;

use crate::nodes::*;

#[derive(Default)]
pub struct NodePrototype {
    pub transform: NodeTransform,
    pub events: EventReceiver,
    pub children: Scene,
}

impl NodePrototype {
    /// take ownership of the prototype
    pub fn take(&mut self) -> Self {
        Self {
            transform: std::mem::take(&mut self.transform),
            children: std::mem::take(&mut self.children),
            events: std::mem::take(&mut self.events),
        }
    }
}

pub trait Builder {
    type Node: Node;

    fn prototype(&mut self) -> &mut NodePrototype;
    fn build(&mut self) -> Self::Node;

    fn transform(&mut self, transform: NodeTransform) -> &mut Self {
        self.prototype().transform = transform;
        self
    }

    fn position(&mut self, position: math::Vec3) -> &mut Self {
        self.prototype().transform.position = position;
        self
    }

    fn rotation(&mut self, rotation: math::Quat) -> &mut Self {
        self.prototype().transform.rotation = rotation;
        self
    }

    fn rotation_euler_xyz(&mut self, rotation: math::Vec3) -> &mut Self {
        self.prototype().transform.set_euler_xyz(rotation);
        self
    }

    fn scale(&mut self, scale: math::Vec3) -> &mut Self {
        self.prototype().transform.scale = scale;
        self
    }

    fn scale_factor(&mut self, scale_factor: f32) -> &mut Self {
        self.prototype().transform.scale *= scale_factor;
        self
    }

    fn on<F>(&mut self, event: Event, callback: F) -> &mut Self
    where
        F: FnMut(&mut Self::Node, &mut GameContext) + 'static,
    {
        self.prototype().events.on(event, callback);
        self
    }

    fn add_child<T: Node>(&mut self, name: &str, child: T) -> &mut Self {
        self.prototype().children.add(name, child);
        self
    }
}

pub trait Buildable {
    type Builder: Builder<Node = Self>;

    fn builder() -> Self::Builder;
}

//todo:
// I thought maybe it would be good to wrap a callback inside a predefined callback that way when the user defines a callback inside of nodebuilder they dont have to worry about downcasting and is added automatically by the NodeBuilder
// since the prototype EventHandler struct needs to call it with dyn node

/// NodeBuilder helps build nodes
pub struct NodeBuilder<T> {
    /// the node being built
    pub node: T,
    /// the nodes children
    pub children: Scene,
    /// the transform of the node
    pub transform: NodeTransform,
    /// the events of the node
    pub events: EventReceiver,
}

impl<T> NodeBuilder<T>
where
    T: Node + Clone,
{
    /// create a new NodeBuilder for the given node
    ///
    /// if the node has a node specific trait such as ModelBuilder then you should use its create
    /// method instead
    ///
    /// # Arguements
    /// - `node` - the node object to create
    ///
    /// # Returns
    /// a nodebuilder for that node
    pub fn new(node: T) -> Self {
        NodeBuilder {
            node,
            children: Scene::default(),
            transform: NodeTransform::default(),
            events: EventReceiver::default(),
        }
    }

    /// add a transform to a node
    pub fn with_transform(&mut self, transform: NodeTransform) -> &mut Self {
        self.transform = transform;
        self
    }

    /// sets the position of the node
    pub fn with_position(&mut self, position: math::Vec3) -> &mut Self {
        self.transform.set_position(position);
        self
    }

    /// sets the rotation of the node
    ///
    /// see [with_rotation_euler_xyz] to rotate with angles
    pub fn with_rotation(&mut self, rotation: math::Quat) -> &mut Self {
        self.transform.set_rotation(rotation);
        self
    }

    /// sets the rotation of the node using euler angles in the xyz order
    pub fn with_rotation_euler_xyz(&mut self, rotation: math::Vec3) -> &mut Self {
        self.transform.rotate_euler_xyz(rotation);
        self
    }

    /// set the scale of the object
    pub fn with_scale(&mut self, scale: math::Vec3) -> &mut Self {
        self.transform.set_scale(scale);
        self
    }

    /// set the scale factor (this with scale xyz uniformly)
    ///
    /// scales all 3 components (xyz) with the same value
    pub fn with_scale_factor(&mut self, scale_factor: f32) -> &mut Self {
        self.transform
            .set_scale(math::vec3(scale_factor, scale_factor, scale_factor));
        self
    }

    /// add a child node
    ///
    /// child nodes transforms are relative to their parents e.g. if a parents positon is (0, 1, 0)
    /// and a childs position is (0, 0, 0) then the world positon would be (0, 1, 0)
    pub fn add_child<U: Node>(&mut self, name: &str, node: U) -> &mut Self {
        let _ = self.children.add(name, node);
        self
    }

    /// add a event to the node
    ///
    /// # Arguements
    /// - `event` - the event schedule e.g. Ready which is ran on start or Update which is ran
    ///     every frame
    /// - `callback` - the function that is ran on that event    
    ///
    pub fn on<F>(&mut self, event: Event, callback: F) -> &mut Self
    where
        F: FnMut(&mut T, &mut GameContext) + 'static,
    {
        self.events.on(event, callback);
        self
    }

    /// build the nodes
    ///
    /// should be ran last when you are done configuring the node.
    ///
    /// # Returns
    /// The built node
    pub fn build(&mut self) -> T
    where
        T: Node + Clone,
    {
        *self.node.get_children_mut() = self.children.clone();
        //println!("{:?}", self.node.get_transform());
        *self.node.get_transform() = self.transform;
        //println!("{:?}", self.node.get_transform());
        *self.node.get_events() = self.events.clone();
        self.node.clone()
        //println!("{:?}", clone.get_transform());
    }
}
