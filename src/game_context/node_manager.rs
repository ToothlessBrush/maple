//! The `node_manager` module defines and manages the scene tree, enabling hierarchical organization and updates for game objects.
//!
//! ## Features
//! - **Node Traits**: `Node`, `Ready`, and `Behavior` for defining custom nodes with initialization and per-frame logic.
//! - **Transformations**: `NodeTransform` struct for position, rotation, scale, and model matrix handling.
//! - **Scene Management**: `NodeManager` for managing child nodes and recursive scene updates.
//!
//! ## Usage
//! Implement the `Node` trait for custom objects, with optional `Ready` and `Behavior` traits for setup and updates. Use `NodeManager` to manage child nodes and relationships.
//!
//! ### Example
//! ```rust
//! //implement the Node trait for a custom node with Ready and Behavior traits
//! use quaturn::engine::game_context::node_manager::{Node, NodeTransform, NodeManager, Ready, Behavior};
//! struct CustomNode {
//!     transform: NodeTransform,
//!     children: NodeManager,
//!     /* more optional fields */
//! }
//! impl Node for CustomNode {
//!    /* ... */
//! }
//! impl Ready for CustomNode {
//!     fn ready(&mut self) {
//!         println!("Node ready!");
//!     }
//! }
//! impl Behavior for CustomNode {
//!     fn behavior(&mut self, _ctx: &mut GameContext) {
//!         println!("Node update!");
//!     }
//! }
//!
//! // add an instance of the custom node to the engine
//!
//! let mut engine = Engine::init("Example", 800, 600);
//!
//! engine.context.nodes.add("custom", CustomNode::new());
//! ```

use super::nodes::{
    camera::Camera3D, directional_light::DirectionalLight, empty::Empty, model::Model, ui::UI,
};
use crate::renderer::shader::Shader;
use egui_gl_glfw::egui::util::id_type_map::SerializableAny;
use nalgebra_glm::{self as glm, Mat4, Vec3};
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// The Ready trait is used to define that has behavior that is called when the node is ready.
///
/// This is useful for nodes that need to perform some kind of setup before the game starts.
///
/// # Example
/// ```rust
/// use quaturn::engine::game_context::node_manager::{Node, NodeTransform, NodeManager, Ready};
/// use std::any::Any;
///
/// struct CustomNode {
///    transform: NodeTransform,
///    children: NodeManager,
///    /* more optional fields */
/// }
///
/// impl Node for CustomNode {
///     fn get_transform(&mut self) -> &mut NodeTransform {
///         &mut self.transform
///     }
///
///     fn get_children(&mut self) -> &mut NodeManager {
///         &mut self.children
///     }
///
///     fn as_any(&self) -> &dyn Any {
///         self
///     }
///
///     fn as_any_mut(&mut self) -> &mut dyn Any {
///         self
///     }
///     // nodes that implement the Ready trait need to have a as_ready method to
///     // cast to the dyn Ready object so the engine can dynamically dispatch the ready method
///     fn as_ready(&mut self) -> Option<&mut (dyn Ready + 'static)> {
///         Some(self)
///     }
/// }
///
/// impl Ready for CustomNode {
///     fn ready(&mut self) {
///         println!("CustomNode is ready!");
///     }
/// }
///
/// impl CustomNode {
///     pub fn new() -> Self {
///         Self {
///             transform: NodeTransform::default(),
///             children: NodeManager::new(),
///        }
///    }
/// }
/// ```
pub trait Ready: Node {
    fn ready(&mut self);
}

/// The Behavior trait is used to define that has behavior that is called every frame.
///
/// This is useful for nodes that need to perform some kind of logic every frame.
///
/// # Example
/// ```rust
/// use quaturn::engine::game_context::node_manager::{Node, NodeTransform, NodeManager, Behavior};
/// use quaturn::engine::game_context::GameContext;
/// use std::any::Any;
///
/// struct CustomNode {
///    transform: NodeTransform,
///    children: NodeManager,
///    /* more optional fields */
/// }
///
/// impl Node for CustomNode {
///     fn get_transform(&mut self) -> &mut NodeTransform {
///         &mut self.transform
///     }
///
///     fn get_children(&mut self) -> &mut NodeManager {
///         &mut self.children
///     }
///
///     fn as_any(&self) -> &dyn Any {
///         self
///     }
///
///     fn as_any_mut(&mut self) -> &mut dyn Any {
///         self
///     }
///     // nodes that implement the Behavior trait need to have a as_behavior method to
///     // cast to the dyn Behavior object so the engine can dynamically dispatch the ready method
///     fn as_behavior(&mut self) -> Option<&mut (dyn Behavior + 'static)> {
///         Some(self)
///     }
/// }
///
/// impl Behavior for CustomNode {
///     fn behavior(&mut self, context: &mut GameContext) {
///         println!("CustomNode is ready!");
///     }
/// }
///
/// impl CustomNode {
///     pub fn new() -> Self {
///         Self {
///             transform: NodeTransform::default(),
///             children: NodeManager::new(),
///        }
///    }
/// }
/// ```
pub trait Behavior: Node {
    fn behavior(&mut self, context: &mut super::GameContext);
}

/// Represents a nodes transform data in 3d space with position, rotation, and scale as well as a precalculated model matrix.
#[derive(Clone)]
pub struct NodeTransform {
    /// position in 3D space with y as up.
    pub position: Vec3,
    /// rotation in quaternion form.
    pub rotation: glm::Quat,
    /// scale in 3D space.
    pub scale: Vec3,
    /// precalculated model matrix.
    pub matrix: Mat4,
}

impl Default for NodeTransform {
    /// the default constructor for NodeTransform sets the position to (0, 0, 0), rotation to identity, scale to (1, 1, 1), and matrix to identity.
    fn default() -> Self {
        let mut transform = Self {
            position: glm::vec3(0.0, 0.0, 0.0),
            rotation: glm::quat_identity(),
            scale: glm::vec3(1.0, 1.0, 1.0),
            matrix: glm::identity(),
        };
        transform.update_matrix();
        transform
    }
}

impl PartialEq for NodeTransform {
    /// compares two NodeTransforms by their position, rotation, scale, and matrix.
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
            && self.rotation == other.rotation
            && self.scale == other.scale
            && self.matrix == other.matrix
    }
}

impl std::fmt::Debug for NodeTransform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Position: {:?}, Rotation: {:?}, Scale: {:?}",
            self.position, self.rotation, self.scale
        )
    }
}

impl NodeTransform {
    /// constructs a new NodeTransform with the given position, rotation, and scale.
    ///
    /// # Arguments
    /// - `position` - the position in 3D space.
    /// - `rotation` - the rotation in quaternion form.
    /// - `scale` - the scale in 3D space.
    ///
    /// # Returns
    /// a new NodeTransform with the given position, rotation, and scale.
    pub fn new(position: Vec3, rotation: glm::Quat, scale: Vec3) -> Self {
        let mut transform = Self {
            position,
            rotation,
            scale,
            matrix: glm::identity(),
        };
        transform.update_matrix();
        transform
    }

    /// updates the model matrix based on the position, rotation, and scale.
    fn update_matrix(&mut self) {
        self.matrix = glm::translation(&self.position)
            * glm::quat_to_mat4(&self.rotation)
            * glm::scaling(&self.scale);
    }

    /// gets the position of the transform.
    ///
    /// # Returns
    /// the position in 3D space.
    pub fn get_position(&self) -> Vec3 {
        self.position
    }

    /// sets the position of the transform.
    ///
    /// # Arguments
    /// - `position` - the new position in 3D space.
    ///
    /// # Returns
    /// a mutable reference to the NodeTransform.
    pub fn set_position(&mut self, position: Vec3) -> &mut Self {
        self.position = position;
        self.update_matrix();
        self
    }

    /// gets the rotation of the transform.
    ///
    /// # Returns
    /// the rotation in quaternion form.
    pub fn get_rotation(&self) -> glm::Quat {
        self.rotation
    }

    /// gets the rotation of the transform as euler angles in degrees.
    ///
    /// # Returns
    /// the rotation as euler angles in degrees.
    pub fn get_rotation_euler_xyz(&self) -> Vec3 {
        glm::quat_euler_angles(&self.rotation)
    }

    /// sets the rotation of the transform.
    ///
    /// # Arguments
    /// - `rotation` - the new rotation in quaternion form.
    ///
    /// # Returns
    /// a mutable reference to the NodeTransform.
    pub fn set_rotation(&mut self, rotation: glm::Quat) -> &mut Self {
        self.rotation = rotation;
        self.update_matrix();
        self
    }

    /// sets the rotation of the transform as euler angles in degrees in xyz order.
    ///
    /// # Arguments
    /// - `degrees` - the new rotation as euler angles in degrees.
    ///
    /// # Returns
    /// a mutable reference to the NodeTransform.
    pub fn set_euler_xyz(&mut self, degrees: Vec3) -> &mut Self {
        let radians = glm::radians(&degrees);
        self.rotation = glm::quat_angle_axis(radians.x, &glm::vec3(1.0, 0.0, 0.0))
            * glm::quat_angle_axis(radians.y, &glm::vec3(0.0, 1.0, 0.0))
            * glm::quat_angle_axis(radians.z, &glm::vec3(0.0, 0.0, 1.0));
        self.update_matrix();
        self
    }

    /// gets the scale of the transform.
    ///
    /// # Returns
    /// the scale in 3D space.
    pub fn get_scale(&self) -> Vec3 {
        self.scale
    }

    /// sets the scale of the transform.
    /// # Arguments
    /// - `scale` - the new scale in 3D space.
    ///
    /// # Returns
    /// a mutable reference to the NodeTransform.
    pub fn set_scale(&mut self, scale: Vec3) -> &mut Self {
        self.scale = scale;
        self.update_matrix();
        self
    }

    /// gets the forward vector of the transform.
    ///
    /// # Returns
    /// the forward vector of the transform.
    pub fn get_forward_vector(&self) -> Vec3 {
        glm::quat_rotate_vec3(&self.rotation, &glm::vec3(0.0, 0.0, 1.0))
    }

    /// gets the right vector of the transform.
    ///
    /// # Returns
    /// the right vector of the transform.
    pub fn get_right_vector(&self) -> Vec3 {
        glm::quat_rotate_vec3(&self.rotation, &glm::vec3(1.0, 0.0, 0.0))
    }

    /// gets the up vector of the transform.
    ///
    /// # Returns
    /// the up vector of the transform.
    pub fn get_up_vector(&self) -> Vec3 {
        glm::quat_rotate_vec3(&self.rotation, &glm::vec3(0.0, 1.0, 0.0))
    }

    /// scales the transform by the given scale.
    ///
    /// # Arguments
    /// - `scale` - the scale to multiply the current scale by.
    ///
    /// # Returns
    /// a mutable reference to the NodeTransform.
    pub fn scale(&mut self, scale: Vec3) -> &mut Self {
        self.scale.x *= scale.x;
        self.scale.y *= scale.y;
        self.scale.z *= scale.z;
        self.update_matrix();
        self
    }

    /// translates the position of the transform by the given translation.
    ///
    /// # Arguments
    /// - `translation` - the translation to add to the current position.
    ///
    /// # Returns
    /// a mutable reference to the NodeTransform.
    pub fn translate(&mut self, translation: Vec3) -> &mut Self {
        self.position += translation;
        self.update_matrix();
        self
    }

    /// rotates the transform by the given axis and degrees.
    ///
    /// # Arguments
    /// - `axis` - the axis to rotate around.
    /// - `degrees` - the degrees to rotate by.
    ///
    /// # Returns
    /// a mutable reference to the NodeTransform.
    pub fn rotate(&mut self, axis: glm::Vec3, degrees: f32) -> &mut Self {
        self.rotation =
            glm::quat_angle_axis(glm::radians(&glm::vec1(degrees)).x, &axis) * self.rotation;
        self.update_matrix();
        self
    }

    /// rotates the transform by the given euler angles in degrees in xyz order.
    ///
    /// # Arguments
    /// - `degrees` - the euler angles in degrees to rotate by.
    ///
    /// # Returns
    /// a mutable reference to the NodeTransform.
    pub fn rotate_euler_xyz(&mut self, degrees: Vec3) -> &mut Self {
        let radians = glm::radians(&degrees);
        self.rotation = glm::quat_angle_axis(radians.x, &glm::vec3(1.0, 0.0, 0.0))
            * glm::quat_angle_axis(radians.y, &glm::vec3(0.0, 1.0, 0.0))
            * glm::quat_angle_axis(radians.z, &glm::vec3(0.0, 0.0, 1.0))
            * self.rotation;
        self.update_matrix();
        self
    }
}

// pub trait Casts: Any {
//     fn as_any(&self) -> &dyn Any;
//     fn as_any_mut(&mut self) -> &mut dyn Any;
// }

// impl<T: Any> Casts for T {
//     fn as_any(&self) -> &dyn Any {
//         self
//     }

//     fn as_any_mut(&mut self) -> &mut dyn Any {
//         self
//     }
// }

// TODO: Implement a more efficient way to cast to a specific trait

/// The Transformable trait is used to define that a node can be transformed.
pub trait Transformable {
    /// applies a transformation to the node and all of its children.
    ///
    /// # Arguments
    /// - `operation` - the operation to apply to the node and all of its children.
    ///
    /// # Returns
    /// a mutable reference to the node.
    fn apply_transform<F>(&mut self, operation: &mut F) -> &mut Self
    where
        F: FnMut(&mut NodeTransform);
}

// implement the Transformable trait for all types that implement the Node trait
impl<T: Node> Transformable for T {
    fn apply_transform<F>(&mut self, mut operation: &mut F) -> &mut Self
    where
        F: FnMut(&mut NodeTransform),
    {
        operation(self.get_transform());
        if let Some(model) = self.as_any_mut().downcast_mut::<Model>() {
            for node in &mut model.nodes {
                operation(&mut node.transform);
            }
        }

        for child in self.get_children().get_all_mut().values_mut() {
            let child_node: &mut dyn Node = &mut **child;
            apply_transform(child_node, operation);
        }
        self
    }
}

/// function that applies a transformation to a node and all of its children.
///
/// # Arguments
/// - `node` - the node to apply the transformation to.
/// - `operation` - the operation to apply to the node and all of its children.
pub fn apply_transform<F>(node: &mut dyn Node, operation: &mut F)
where
    F: FnMut(&mut NodeTransform),
{
    operation(node.get_transform());

    if let Some(model) = node.as_any_mut().downcast_mut::<Model>() {
        for node in &mut model.nodes {
            operation(&mut node.transform);
        }
    }

    for child in node.get_children().get_all_mut().values_mut() {
        let child_node: &mut dyn Node = &mut **child;
        apply_transform(child_node, operation);
        println!("processing children");
    }
}

/// The Casting trait is used to define that a type can be cast to Any.
pub trait Casting {
    /// cast to Any trait object.
    fn as_any(&self) -> &dyn Any;
    /// cast to mutable Any trait object.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// blanket implementation of the Casting trait for all types that implement the Node trait.
impl<T: Node> Casting for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// The Node trait is used to define that a type is a node in the scene graph.
/// A node is a part of the scene tree that can be transformed and have children.
/// the node_manager only stores nodes that implement the Node trait.
pub trait Node: Any + Casting {
    /// gets the model matrix of the node.
    ///
    /// # Returns
    /// the model matrix of the node.
    fn get_model_matrix(&mut self) -> &glm::Mat4 {
        &self.get_transform().matrix
    }

    /// gets the transform of the node.
    ///
    /// # Returns
    /// a mutable reference to the transform of the node.
    fn get_transform(&mut self) -> &mut NodeTransform;

    /// gets the children of the node.
    ///
    /// # Returns
    /// a mutable reference to the children of the node.
    fn get_children(&mut self) -> &mut NodeManager;

    /// Cast to Ready trait if it implements it
    ///
    /// A node that implements the Ready trait need to have a as_ready method to cast to the dyn Ready object so the engine can dynamically dispatch the ready method
    fn as_ready(&mut self) -> Option<&mut dyn Ready> {
        None
    }

    /// cast to Behavior trait if it implements it
    ///
    /// A node that implements the Behavior trait need to have a as_behavior method to cast to the dyn Behavior object so the engine can dynamically dispatch the behavior method
    fn as_behavior(&mut self) -> Option<&mut dyn Behavior> {
        None
    }
}

/// The Drawable trait is used to define that a type can be drawn.
pub trait Drawable {
    /// draws the object using the given shader and camera.
    ///
    /// # Arguments
    /// - `shader` - the shader to use to draw the object.
    /// - `camera` - the camera to use to draw the object.
    fn draw(&mut self, shader: &mut Shader, camera: &Camera3D);
    /// draws the object using the given shader and light space matrix for rendering a depth map from the lights perspective.
    ///
    /// # Arguments
    /// - `shader` - the shader to use to draw the object.
    /// - `light_space_matrix` - the light space matrix to use to draw the object.
    fn draw_shadow(&mut self, shader: &mut Shader, light_space_matrix: &Mat4);
}

/// The NodeManager struct is used to manage all the nodes in the scene tree.
pub struct NodeManager {
    /// A hashmap of all the nodes in the scene tree.
    nodes: HashMap<String, Box<dyn Node>>,
    /// A hashmap of all the shaders in the scene.
    pub shaders: HashMap<String, Box<Shader>>,
    /// The shadow shader used to render depth maps.
    pub shadow_shader: Option<Shader>,
    /// The active camera in the scene.
    pub active_camera: String,
    /// The active shader in the scene.
    pub active_shader: String,
}

impl Default for NodeManager {
    /// the default constructor for NodeManager creates a new NodeManager with no nodes, shaders, or active camera.
    fn default() -> Self {
        Self::new()
    }
}

impl NodeManager {
    /// constructs a new NodeManager with no nodes, shaders, or active camera.
    pub fn new() -> NodeManager {
        NodeManager {
            nodes: HashMap::new(),
            shaders: HashMap::new(),
            active_camera: String::new(),
            active_shader: String::new(),
            shadow_shader: None,
        }
    }

    /// adds a node to the scene tree with the given name.
    ///
    /// # Arguments
    /// - `name` - the name of the node.
    /// - `node` - the node to add to the scene tree.
    ///
    /// # Returns
    /// a mutable reference to the node.
    ///
    /// # Panics
    /// if the node cannot be downcast to the given type.
    ///
    /// # Example
    /// ```rust
    /// use quaturn::game_context::nodes::empty::Empty;
    /// use quaturn::Engine;
    /// use std::any::Any;
    ///
    /// let mut engine = Engine::init("Example", 800, 600);
    ///
    /// engine.context.nodes.add("empty", Empty::new());
    /// ```
    pub fn add<T: Node + 'static>(&mut self, name: &str, node: T) -> &mut T {
        // Insert the node into the map
        self.nodes.insert(name.to_string(), Box::new(node));

        // If it's the first camera added, set it as the active camera
        if std::any::type_name::<T>() == std::any::type_name::<Camera3D>()
            && self.active_camera.is_empty()
        {
            self.active_camera = name.to_string();
        }

        // Safely downcast and return the node
        self.nodes
            .get_mut(name)
            .and_then(|node| node.as_any_mut().downcast_mut::<T>())
            .expect("Failed to downcast the node")
    }

    /// runs the ready method if the node implements the Ready trait and reruns this method for children.
    pub fn ready(&mut self) {
        for node in self.nodes.values_mut() {
            if let Some(node) = node.as_ready() {
                node.ready();
            }
            // recursively call ready on all children
            node.get_children().ready();
        }
    }

    /// runs the behavior method if the node implements the Behavior trait and reruns this method for children.
    pub fn behavior(&mut self, context: &mut super::GameContext) {
        for node in self.nodes.values_mut() {
            if let Some(node) = node.as_behavior() {
                node.behavior(context);
            }
            // recursively call behavior on all children
            node.get_children().behavior(context);
        }
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

    /// get a node by name.
    ///
    /// # Arguments
    /// - `name` - the name of the node.
    ///
    /// # Returns
    /// a reference to the node.
    pub fn get<T: 'static + Node>(&self, name: &str) -> Option<&T> {
        self.nodes
            .get(name)
            .and_then(|node| node.as_any().downcast_ref::<T>())
    }

    /// get a mutable reference to a node by name.
    ///
    /// # Arguments
    /// - `name` - the name of the node.
    ///
    /// # Returns
    /// a mutable reference to the node.
    pub fn get_mut<T: 'static + Node>(&mut self, name: &str) -> Option<&mut T> {
        self.nodes
            .get_mut(name)
            .and_then(|node| node.as_any_mut().downcast_mut::<T>())
    }

    /// get all nodes of a specific type as an iterator
    ///
    /// # Returns
    /// an iterator of mutable references to all nodes of the given type.
    pub fn get_iter<T: 'static + Node>(&mut self) -> impl Iterator<Item = &mut T> {
        self.nodes
            .values_mut()
            .filter_map(|node| node.as_any_mut().downcast_mut::<T>())
    }

    /// get all nodes of a specific type as a vector
    ///
    /// # Returns
    /// a vector of mutable references to all nodes of the given type.
    pub fn get_vec<T: 'static + Node>(&mut self) -> Vec<&mut T> {
        self.nodes
            .values_mut()
            .filter_map(|node| node.as_any_mut().downcast_mut::<T>())
            .collect()
    }

    /// add a shader to the scene.
    ///
    /// # Arguments
    /// - `name` - the name of the shader.
    /// - `shader` - the shader to add to the scene.
    ///
    /// # Returns
    /// a mutable reference to the shader.
    ///
    /// # Example
    /// ```rust
    /// /// use quaturn::game_context::nodes::empty::Empty;
    /// use quaturn::renderer::shader::Shader;
    /// use quaturn::Engine;
    /// use std::any::Any;
    ///
    /// let mut engine = Engine::init("Example", 800, 600);
    ///
    /// engine.context.nodes.add_shader("default", Shader::default());
    /// ```
    pub fn add_shader(&mut self, name: &str, shader: Shader) -> &mut Shader {
        self.shaders.insert(name.to_string(), Box::new(shader));
        if self.active_shader.is_empty() {
            self.active_shader = name.to_string();
        }
        self.shaders.get_mut(name).unwrap()
    }
}
