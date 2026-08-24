//! represents the current transform of a given node. each node has a transform that can be manipulated to move, rotate, and scale the node in 3D space.

use glam::{Mat4, Quat, Vec3};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Represents a nodes transform data in 3d space with position, rotation, and scale as well as a precalculated model matrix.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NodeTransform {
    /// position in 3D space with y as up.
    pub position: Vec3,
    /// rotation in quaternion form.
    pub rotation: Quat,
    /// scale in 3D space.
    pub scale: Vec3,
    /// readonly field that stores the nodes position in world space
    #[cfg_attr(feature = "serde", serde(skip))]
    world_transform: WorldTransform,
}

/// represents a position in worldspace
#[derive(Clone, Copy)]
pub struct WorldTransform {
    /// position in worldspace
    pub position: Vec3,
    /// rotation in worldspace
    pub rotation: Quat,
    /// scale in worldspace
    pub scale: Vec3,
    /// matrix of the world position
    pub matrix: Mat4,
}

impl Default for WorldTransform {
    fn default() -> Self {
        let mut out = Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            matrix: Mat4::IDENTITY,
        };

        out.update_matrix();
        out
    }
}

impl std::ops::Add for NodeTransform {
    type Output = NodeTransform;

    fn add(self, rhs: Self) -> Self::Output {
        let rotated_position = self.rotation * rhs.position; // position relative to parent space
        let position = self.position + rotated_position * self.scale; // scale relative to parent space scale
        let rotation = (self.rotation * rhs.rotation).normalize();
        let scale = self.scale * rhs.scale;

        Self::new(position, rotation, scale)
    }
}

// same thing but for world transform
impl std::ops::Add for WorldTransform {
    type Output = WorldTransform;

    fn add(self, rhs: Self) -> Self::Output {
        let rotated_position = self.rotation * rhs.position; // position relative to parent space
        let position = self.position + rotated_position * self.scale; // scale relative to parent space scale
        let rotation = (self.rotation * rhs.rotation).normalize();
        let scale = self.scale * rhs.scale;

        let mut result = Self {
            position,
            rotation,
            scale,
            matrix: Mat4::IDENTITY,
        };

        result.update_matrix();
        result
    }
}

impl WorldTransform {
    /// updates the model matrix based on the position, rotation, and scale.
    fn update_matrix(&mut self) {
        self.matrix = Mat4::from_scale_rotation_translation(
            self.scale,
            self.rotation.normalize(),
            self.position,
        );
    }
}

impl Default for NodeTransform {
    /// the default constructor for NodeTransform sets the position to (0, 0, 0), rotation to identity, scale to (1, 1, 1), and matrix to identity.
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            world_transform: WorldTransform::default(),
        }
    }
}

impl PartialEq for NodeTransform {
    /// compares two NodeTransforms by their position, rotation, scale, and matrix.
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
            && self.rotation == other.rotation
            && self.scale == other.scale
    }
}

impl std::fmt::Debug for NodeTransform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Local => Position: {:?}, Rotation: {:?}, Scale: {:?}\n\
             World => Position: {:?}, Rotation: {:?}, Scale: {:?}",
            self.position,
            self.rotation,
            self.scale,
            self.world_transform.position,
            self.world_transform.rotation,
            self.world_transform.scale
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
    pub fn new(position: impl Into<Vec3>, rotation: Quat, scale: impl Into<Vec3>) -> Self {
        Self {
            position: position.into(),
            rotation,
            scale: scale.into(),
            world_transform: WorldTransform::default(),
        }
    }

    /// returns the world space of the object
    ///
    /// this is not meant to be modified and will not update when you modify localspace
    #[inline]
    pub fn world_space(&self) -> &WorldTransform {
        &self.world_transform
    }

    /// get the world space transform of the transform
    ///
    /// useful if you need to know where a node is in the world
    pub fn update_world_space(&mut self, parent_space: WorldTransform) {
        // we need to add self to the worldspace to get the current objects worldspace
        // the current worldspace is considered dirty so we cant use self.worldspace as this is
        // called after localspace has been modified
        let local_world_space = WorldTransform {
            position: self.position,
            rotation: self.rotation,
            scale: self.scale,
            matrix: Mat4::IDENTITY,
        };

        self.world_transform = parent_space + local_world_space;
        self.world_transform.update_matrix();
    }

    /// gets the position of the transform.
    ///
    /// # Returns
    /// the position in 3D space.
    pub fn get_position(&self) -> &Vec3 {
        &self.position
    }

    /// gets a mutible position
    pub fn get_position_mut(&mut self) -> &mut Vec3 {
        &mut self.position
    }

    /// linarly interpolate the transform between 2 transforms and a t value
    pub fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        let position = a.position.lerp(b.position, t);
        let rotation = a.rotation.slerp(b.rotation, t);
        let scale = a.scale.lerp(b.scale, t);

        Self::new(position, rotation, scale)
    }

    /// sets the position of the transform.
    ///
    /// # Arguments
    /// - `position` - the new position in 3D space.
    ///
    /// # Returns
    /// a mutable reference to the NodeTransform.
    pub fn set_position(&mut self, position: impl Into<Vec3>) -> &mut Self {
        self.position = position.into();
        self
    }

    /// gets the rotation of the transform.
    ///
    /// # Returns
    /// the rotation in quaternion form.
    pub fn get_rotation(&self) -> &Quat {
        &self.rotation
    }

    /// returns a mutible refrence to the rotation quat
    pub fn get_rotation_mut(&mut self) -> &mut Quat {
        &mut self.rotation
    }

    /// gets the rotation of the transform as euler angles in degrees.
    ///
    /// # Returns
    /// the rotation as euler angles in degrees.
    pub fn get_rotation_euler_xyz_degrees(&self) -> Vec3 {
        let (x, y, z) = self.rotation.to_euler(glam::EulerRot::XYZ);
        Vec3::new(x.to_degrees(), y.to_degrees(), z.to_degrees())
    }

    /// gets the rotation of the transform as euler angles in radians.
    ///
    /// # Returns
    /// the rotation as euler angles in degrees.
    pub fn get_rotation_euler_xyz_radians(&self) -> Vec3 {
        let (x, y, z) = self.rotation.to_euler(glam::EulerRot::XYZ);
        Vec3::new(x, y, z)
    }

    /// sets the rotation of the transform.
    ///
    /// # Arguments
    /// - `rotation` - the new rotation in quaternion form.
    ///
    /// # Returns
    /// a mutable reference to the NodeTransform.
    pub fn set_rotation(&mut self, rotation: Quat) -> &mut Self {
        self.rotation = rotation;
        self
    }

    /// sets the rotation of the transform as euler angles in degrees in xyz order.
    pub fn set_euler_xyz_degrees(&mut self, degrees: impl Into<Vec3>) -> &mut Self {
        let degrees = degrees.into();

        self.rotation = Quat::from_euler(
            glam::EulerRot::XYZ,
            degrees.x.to_radians(),
            degrees.y.to_radians(),
            degrees.z.to_radians(),
        )
        .normalize();
        self
    }

    /// sets the rotation of the transform as euler angles in degrees in xyz order.
    pub fn set_euler_xyz_radians(&mut self, degrees: impl Into<Vec3>) -> &mut Self {
        let degrees = degrees.into();

        self.rotation =
            Quat::from_euler(glam::EulerRot::XYZ, degrees.x, degrees.y, degrees.z).normalize();
        self
    }

    /// gets the scale of the transform.
    ///
    /// # Returns
    /// the scale in 3D space.
    pub fn get_scale(&self) -> &Vec3 {
        &self.scale
    }

    /// get a mutible refrence to the scale
    pub fn get_scale_mut(&mut self) -> &mut Vec3 {
        &mut self.scale
    }

    /// sets the scale of the transform.
    /// # Arguments
    /// - `scale` - the new scale in 3D space.
    ///
    /// # Returns
    /// a mutable reference to the NodeTransform.
    pub fn set_scale(&mut self, scale: impl Into<Vec3>) -> &mut Self {
        self.scale = scale.into();
        self
    }

    pub fn set_scale_factor(&mut self, scale: f32) -> &mut Self {
        self.scale = self.scale * scale;
        self
    }

    /// gets the forward vector of the transform.
    ///
    /// # Returns
    /// the forward vector of the transform.
    pub fn get_forward_vector(&self) -> Vec3 {
        self.rotation * Vec3::NEG_Z
    }

    /// gets the right vector of the transform.
    ///
    /// # Returns
    /// the right vector of the transform.
    pub fn get_right_vector(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    /// gets the up vector of the transform.
    ///
    /// # Returns
    /// the up vector of the transform.
    pub fn get_up_vector(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    /// scales the transform by the given scale.
    ///
    /// # Arguments
    /// - `scale` - the scale to multiply the current scale by.
    ///
    /// # Returns
    /// a mutable reference to the NodeTransform.
    pub fn scale(&mut self, scale: impl Into<Vec3>) -> &mut Self {
        self.scale *= scale.into();
        self
    }

    /// translates the position of the transform by the given translation.
    ///
    /// # Arguments
    /// - `translation` - the translation to add to the current position.
    ///
    /// # Returns
    /// a mutable reference to the NodeTransform.
    pub fn translate(&mut self, translation: impl Into<Vec3>) -> &mut Self {
        self.position += translation.into();
        self
    }

    /// translates the position of the transform by the given translation in world space.
    /// This ignores the objects rotation when moving,
    ///
    /// # Arguments
    /// - `translation` - the translation to add to the current position.
    pub fn translate_world_space(&mut self, translation: impl Into<Vec3>) -> &mut Self {
        self.position += self.rotation * translation.into();
        self
    }

    /// rotates the transform by the given axis and degrees.
    pub fn rotate_degrees(&mut self, axis: impl Into<Vec3>, degrees: f32) -> &mut Self {
        let angle_quat = Quat::from_axis_angle(axis.into().normalize(), degrees.to_radians());
        self.rotation = angle_quat * self.rotation;
        self
    }

    /// rotates the transform by the given axis and radians
    pub fn rotate_radians(&mut self, axis: impl Into<Vec3>, radians: f32) -> &mut Self {
        let angle_quat = Quat::from_axis_angle(axis.into().normalize(), radians);
        self.rotation = angle_quat * self.rotation;
        self
    }

    pub fn looking_at(&mut self, target: impl Into<Vec3>) -> &mut Self {
        self.set_orientation_vector(target.into() - self.position);
        self
    }

    pub fn set_orientation_vector(&mut self, orientation: impl Into<Vec3>) -> &mut Self {
        let orientation = orientation.into().normalize();
        let default_forward = Vec3::NEG_Z;

        if orientation == default_forward {
            self.set_rotation(Quat::IDENTITY);
            return self;
        }

        let rotation_axis = default_forward.cross(orientation);

        // Handle anti-parallel case (orientation opposite to default forward)
        if rotation_axis.length_squared() < 0.0001 {
            // Vectors are anti-parallel, rotate 180 degrees around Y-axis
            let rotation_quat =
                Quat::from_axis_angle(glam::vec3(0.0, 1.0, 0.0), std::f32::consts::PI);
            self.set_rotation(rotation_quat);
            return self;
        }

        let rotation_axis = rotation_axis.normalize();
        let rotation_angle = default_forward.dot(orientation).acos();
        let rotation_quat = Quat::from_axis_angle(rotation_axis, rotation_angle);
        self.set_rotation(rotation_quat);

        self
    }

    /// rotates the transform by the given euler angles in degrees in xyz order.
    pub fn rotate_euler_xyz_degrees(&mut self, degrees: impl Into<Vec3>) -> &mut Self {
        let degrees = degrees.into();

        let euler_quat = Quat::from_euler(
            glam::EulerRot::XYZ,
            degrees.x.to_radians(),
            degrees.y.to_radians(),
            degrees.z.to_radians(),
        );
        self.rotation = (euler_quat * self.rotation).normalize();
        self
    }

    /// rotates the transform by the given euler angles in radians in xyz order.
    pub fn rotate_euler_xyz_radians(&mut self, radians: impl Into<Vec3>) -> &mut Self {
        let radians = radians.into();

        let euler_quat = Quat::from_euler(glam::EulerRot::XYZ, radians.x, radians.y, radians.z);
        self.rotation = (euler_quat * self.rotation).normalize();
        self
    }
}

impl From<NodeTransform> for WorldTransform {
    fn from(value: NodeTransform) -> Self {
        let mut out = Self {
            position: value.position,
            rotation: value.rotation,
            scale: value.scale,
            matrix: Mat4::IDENTITY,
        };
        out.update_matrix();
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Quat, Vec3};

    #[test]
    fn test_default_transform() {
        let transform = NodeTransform::default();
        assert_eq!(transform.position, Vec3::ZERO);
        assert_eq!(transform.rotation, Quat::IDENTITY);
        assert_eq!(transform.scale, Vec3::ONE);
    }

    #[test]
    fn test_translation() {
        let mut transform = NodeTransform::default();
        transform.translate(Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(transform.position, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_rotation() {
        let mut transform = NodeTransform::default();
        transform.rotate_degrees(Vec3::Y, 90.0);
        let expected_rotation = Quat::from_axis_angle(Vec3::Y, 90.0_f32.to_radians());
        assert_eq!(transform.rotation, expected_rotation);
    }

    #[test]
    fn test_scaling() {
        let mut transform = NodeTransform::default();
        transform.scale(Vec3::new(2.0, 3.0, 4.0));
        assert_eq!(transform.scale, Vec3::new(2.0, 3.0, 4.0));
    }

    #[test]
    fn test_add_transform() {
        const EPSILON: f32 = 1e-5;

        fn approx_eq(v1: &Vec3, v2: &Vec3) -> bool {
            (*v1 - *v2).length() < EPSILON
        }

        fn approx_eq_quat(q1: &Quat, q2: &Quat) -> bool {
            q1.dot(*q2).abs() > 1.0 - EPSILON
        }

        let transform1 = NodeTransform::new(
            Vec3::new(1.0, 0.0, 0.0),
            Quat::from_axis_angle(Vec3::Y, 90.0_f32.to_radians()),
            Vec3::new(2.0, 2.0, 2.0),
        );

        let transform2 = NodeTransform::new(
            Vec3::new(0.0, 1.0, 0.0),
            Quat::from_axis_angle(Vec3::X, 90.0_f32.to_radians()),
            Vec3::new(0.5, 0.5, 0.5),
        );

        let result = transform1 + transform2;

        let expected_position = Vec3::new(1.0, 2.0, 0.0);
        let expected_rotation = (transform1.rotation * transform2.rotation).normalize();
        let expected_scale = Vec3::new(1.0, 1.0, 1.0);

        assert!(
            approx_eq(&result.position, &expected_position),
            "position: {:?} != {:?}",
            result.position,
            expected_position
        );
        assert!(
            approx_eq_quat(&result.rotation, &expected_rotation),
            "rotation: {:?} != {:?}",
            result.rotation,
            expected_rotation
        );
        assert!(
            approx_eq(&result.scale, &expected_scale),
            "scale: {:?} != {:?}",
            result.scale,
            expected_scale
        );
    }

    #[test]
    fn test_euler_rotation() {
        let mut transform = NodeTransform::default();
        transform.set_euler_xyz_degrees(Vec3::new(90.0, 0.0, 0.0));

        let expected_rotation = Quat::from_axis_angle(Vec3::X, 90.0_f32.to_radians());

        const EPSILON: f32 = 0.001;
        assert!(transform.rotation.angle_between(expected_rotation) < EPSILON);
    }

    #[test]
    fn test_get_euler() {
        let mut transform = NodeTransform::default();
        transform.set_euler_xyz_degrees(Vec3::new(90.0, 0.0, 0.0));

        let result = transform.get_rotation_euler_xyz_degrees();
        let expected = Vec3::new(90.0, 0.0, 0.0);

        // Compare with epsilon
        const EPSILON: f32 = 0.001; // Slightly larger epsilon for euler angle conversion
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
