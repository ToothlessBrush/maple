//! represents the current transform of a given node. each node has a transform that can be manipulated to move, rotate, and scale the node in 3D space.

use glm::{Mat4, Vec3};
use gltf::Node;
use nalgebra_glm as glm;

/// Represents a nodes transform data in 3d space with position, rotation, and scale as well as a precalculated model matrix.
#[derive(Clone, Copy)]
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

impl std::ops::Add for NodeTransform {
    type Output = NodeTransform;

    fn add(self, rhs: Self) -> Self::Output {
        let position = self.position + rhs.position;
        let rotation = glm::quat_normalize(&(self.rotation * rhs.rotation));
        let scale = glm::vec3(
            self.scale.x * rhs.scale.x,
            self.scale.y * rhs.scale.y,
            self.scale.z * rhs.scale.z,
        );

        Self::new(position, rotation, scale)
    }
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
            "Position: {:?}, Rotation: {:?}, Scale: {:?}, Matrix: {:?}",
            self.position, self.rotation, self.scale, self.matrix
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
        // Extract Euler angles in XYZ order
        let (x, y, z) = {
            let (sin_x_cos_y, cos_x_cos_y, sin_y) = {
                let q = self.rotation;
                let sin_y = 2.0 * (q.w * q.j - q.k * q.i);

                // Handle gimbal lock at y = ±90°
                if sin_y.abs() > 0.999 {
                    let sin_x_cos_y = 2.0 * (q.w * q.i + q.j * q.k);
                    let cos_x_cos_y = 1.0 - 2.0 * (q.i * q.i + q.j * q.j);
                    (sin_x_cos_y, cos_x_cos_y, sin_y)
                } else {
                    let sin_x_cos_y = 2.0 * (q.w * q.i + q.j * q.k);
                    let cos_x_cos_y = 1.0 - 2.0 * (q.i * q.i + q.j * q.j);
                    (sin_x_cos_y, cos_x_cos_y, sin_y)
                }
            };

            let x = sin_x_cos_y.atan2(cos_x_cos_y);
            let y = sin_y.asin();
            let z = (2.0 * (self.rotation.w * self.rotation.k + self.rotation.i * self.rotation.j))
                .atan2(
                    1.0 - 2.0
                        * (self.rotation.j * self.rotation.j + self.rotation.k * self.rotation.k),
                );

            (x, y, z)
        };

        glm::vec3(x, y, z)
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

#[cfg(test)]
mod tests {
    use super::*;
    use glm::{quat_identity, vec3, Quat};

    #[test]
    fn test_default_transform() {
        let transform = NodeTransform::default();
        assert_eq!(transform.position, vec3(0.0, 0.0, 0.0));
        assert_eq!(transform.rotation, quat_identity());
        assert_eq!(transform.scale, vec3(1.0, 1.0, 1.0));
        assert_eq!(transform.matrix, glm::Mat4::identity());
    }

    #[test]
    fn test_translation() {
        let mut transform = NodeTransform::default();
        transform.translate(vec3(1.0, 2.0, 3.0));
        assert_eq!(transform.position, vec3(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_rotation() {
        let mut transform = NodeTransform::default();
        transform.rotate(vec3(0.0, 1.0, 0.0), 90.0);
        let expected_rotation =
            glm::quat_angle_axis(glm::radians(&glm::vec1(90.0)).x, &vec3(0.0, 1.0, 0.0));
        assert_eq!(transform.rotation, expected_rotation);
    }

    #[test]
    fn test_scaling() {
        let mut transform = NodeTransform::default();
        transform.scale(vec3(2.0, 3.0, 4.0));
        assert_eq!(transform.scale, vec3(2.0, 3.0, 4.0));
    }

    #[test]
    fn test_model_matrix_update() {
        let mut transform = NodeTransform::default();
        transform.set_position(vec3(1.0, 2.0, 3.0));
        transform.set_scale(vec3(2.0, 2.0, 2.0));
        transform.set_rotation(glm::quat_angle_axis(
            glm::radians(&glm::vec1(45.0)).x,
            &vec3(0.0, 1.0, 0.0),
        ));

        let expected_matrix = glm::translation(&transform.position)
            * glm::quat_to_mat4(&transform.rotation)
            * glm::scaling(&transform.scale);
        assert!(transform.matrix == expected_matrix);
    }

    #[test]
    fn test_add_transform() {
        let transform1 = NodeTransform::new(
            vec3(1.0, 0.0, 0.0),
            glm::quat_angle_axis(glm::radians(&glm::vec1(90.0)).x, &vec3(0.0, 1.0, 0.0)),
            vec3(2.0, 2.0, 2.0),
        );

        let transform2 = NodeTransform::new(
            vec3(0.0, 1.0, 0.0),
            glm::quat_angle_axis(glm::radians(&glm::vec1(90.0)).x, &vec3(1.0, 0.0, 0.0)),
            vec3(0.5, 0.5, 0.5),
        );

        let result = transform1 + transform2;

        let expected_position = vec3(1.0, 1.0, 0.0);
        assert!(result.position == expected_position);

        let expected_rotation = glm::quat_normalize(&(transform1.rotation * transform2.rotation));
        assert!(result.rotation == expected_rotation);

        let expected_scale = vec3(1.0, 1.0, 1.0);
        assert!(result.scale == expected_scale);
    }

    #[test]
    fn test_euler_rotation() {
        let mut transform = NodeTransform::default();
        transform.set_euler_xyz(vec3(90.0, 0.0, 0.0));

        let expected_rotation =
            glm::quat_angle_axis(glm::radians(&glm::vec1(90.0)).x, &vec3(1.0, 0.0, 0.0));
        assert!(transform.rotation == expected_rotation);
    }

    #[test]
    fn test_get_euler() {
        let mut transform = NodeTransform::default();
        transform.set_euler_xyz(vec3(90.0, 0.0, 0.0));

        let result = transform.get_rotation_euler_xyz();
        let expected = glm::radians(&vec3(90.0, 0.0, 0.0));

        // Compare with epsilon
        const EPSILON: f32 = 0.0001;
        assert!(
            (result.x - expected.x).abs() < EPSILON
                && (result.y - expected.y).abs() < EPSILON
                && (result.z - expected.z).abs() < EPSILON,
            "Expected approximately {:?}, got {:?}",
            expected,
            result
        );
    }
}
