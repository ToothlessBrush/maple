//! represents the current transform of a given node. each node has a transform that can be manipulated to move, rotate, and scale the node in 3D space.

use glm::{Mat4, Vec3};
use nalgebra_glm as glm;

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
    pub fn get_position(&self) -> &Vec3 {
        &self.position
    }

    pub fn get_position_mut(&mut self) -> &mut Vec3 {
        &mut self.position
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
    pub fn get_rotation(&self) -> &glm::Quat {
        &self.rotation
    }

    pub fn get_rotation_mut(&mut self) -> &mut glm::Quat {
        &mut self.rotation
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
    pub fn get_scale(&self) -> &Vec3 {
        &self.scale
    }

    pub fn get_scale_mut(&mut self) -> &mut Vec3 {
        &mut self.scale
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

    /// translates the position of the transform by the given translation in world space.
    /// This ignores the objects rotation when moving,
    ///
    /// # Arguments
    /// - `translation` - the translation to add to the current position.
    pub fn translate_world_space(&mut self, translation: Vec3) -> &mut Self {
        self.position += glm::quat_rotate_vec3(&self.rotation, &translation);
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
