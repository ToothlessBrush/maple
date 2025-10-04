//! The Camera node is where the scene is rendered from.
//!
//! ## Usage
//! add a camera node the the scene and set the camera as the main camera in the game context and the engine will render the scene from the camera's perspective.
//!

extern crate glam as math;
use std::f32::consts::FRAC_PI_4;

use bytemuck::{Pod, Zeroable};
use maple_engine::{
    Buildable, Builder, Node, Scene,
    input::{InputManager, KeyCode},
    nodes::node_builder::NodePrototype,
    prelude::{EventReceiver, NodeTransform},
};

#[derive(Default, Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Camera3DBufferData {
    position: [f32; 4],
    view: [[f32; 4]; 4],
    projection: [[f32; 4]; 4],
    vp: [[f32; 4]; 4],
}

/// A 3D camera that can be use in a 3d environment.
pub struct Camera3D {
    /// the NodeTransform of the camera (every node has this)
    pub transform: NodeTransform,
    /// the children of the camera (every node has this)
    pub children: Scene,
    /// events
    pub events: EventReceiver,
    /// the field of view of the camera in radians
    pub fov: f32,
    /// the near plane of the camera
    near: f32,
    /// the far plane of the camera
    far: f32,
    // if the camera is active or not
    pub is_active: bool,
    // if multiple cameras are active it will draw in the order of priority
    pub priority: i32,
}

impl Node for Camera3D {
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }

    fn get_children(&self) -> &Scene {
        &self.children
    }

    fn get_children_mut(&mut self) -> &mut Scene {
        &mut self.children
    }

    fn get_events(&mut self) -> &mut EventReceiver {
        &mut self.events
    }
}

impl Camera3D {
    /// Creates a new 3D camera
    ///
    /// # Arguments
    /// - `position` - The position of the camera
    /// - `orientation` - The orientation vector of the camera (where the camera is looking)
    /// - `fov` - The field of view of the camera
    /// - `aspect_ratio` - The aspect ratio of the camera
    /// - `near` - The near plane of the camera
    /// - `far` - The far plane of the camera
    ///
    /// # Returns
    /// A new Camera3D
    pub fn new(fov: f32, near: f32, far: f32) -> Camera3D {
        Camera3D {
            transform: NodeTransform::default(),
            children: Scene::new(),
            events: EventReceiver::new(),

            fov,
            near,
            far,

            is_active: true,
            priority: 0,
        }
    }

    /// set the orientation of the camera with a vector
    ///
    /// # Arguements
    /// - `orientation` - vector in the direction the camera will look
    pub fn set_orientation(&mut self, orientation: math::Vec3) -> &mut Self {
        let orientation = orientation.normalize();
        // if orientation default then reset quat
        if orientation == math::vec3(0.0, 0.0, 1.0) {
            self.transform.set_rotation(math::Quat::IDENTITY);
            return self;
        }

        let rotation_axis = math::vec3(0.0, 0.0, 1.0).cross(orientation);
        let rotation_angle = math::vec3(0.0, 0.0, 1.0).dot(orientation).acos();
        let rotation_quat = math::Quat::from_axis_angle(rotation_axis, rotation_angle);

        self.transform.set_rotation(rotation_quat);

        self
    }

    /// offset the camera position
    ///
    /// # Arguments
    /// - `offset` - The offset to move the camera by a 3d vector
    pub fn move_camera(&mut self, offset: math::Vec3) {
        //can be used to move the camera around the origin
        self.transform.position += offset;
    }

    /// rotate the camera while keeping the roll at 0
    ///
    /// # Arguments
    /// - `offset` - The offset to rotate the camera by a 3d vector
    /// - `sensitivity` - The sensitivity of the rotation
    pub fn rotate_camera(&mut self, offset: math::Vec3, sensitivity: f32) {
        let max_pitch = 89.90f32.to_radians(); // prevent gimbal lock

        // Calculate pitch and yaw deltas
        let pitch_offset = offset.y * sensitivity;
        let yaw_offset = -offset.x * sensitivity;

        //println!("{}, {}", pitch_offset, yaw_offset);

        // Get the forward vector and calculate the current pitch
        let forward = self.transform.get_forward_vector().normalize();

        let current_pitch = self.get_orientation_angles().y.to_radians();

        // Calculate the target pitch
        let target_pitch = (current_pitch + pitch_offset).clamp(-max_pitch, max_pitch);

        // Limit the pitch delta before applying it
        let clamped_pitch_offset = target_pitch - current_pitch;
        // println!("{}", clamped_pitch_offset); // This should be 0 when the current pitch is at the max_pitch but its not

        // Calculate the right vector
        let right = math::vec3(0.0, 1.0, 0.0).cross(forward).normalize(); // we cant use get_right_vector becuase it needs to be relative to the world up and forward not the camera up and forward

        // Create quaternions for pitch and yaw
        let pitch_quat = math::Quat::from_axis_angle(right, clamped_pitch_offset);
        let yaw_quat = math::Quat::from_axis_angle(math::vec3(0.0, 1.0, 0.0), yaw_offset); // rotate around world up

        // Combine quaternions and apply to the camera
        let combined_quat = yaw_quat * pitch_quat;
        let new_rotation = combined_quat * self.transform.rotation;

        // Normalize the rotation quaternion
        self.transform.set_rotation(new_rotation.normalize());
    }

    /// set the position of the camera
    ///
    /// # Arguments
    /// - `position` - The new position of the camera
    pub fn set_position(&mut self, position: math::Vec3) {
        self.transform.position = position;
    }

    /// get the world space position of the camera
    ///
    /// # Returns
    /// The position of the camera
    pub fn get_position(&self, parent_transform: NodeTransform) -> math::Vec3 {
        (parent_transform + self.transform).position
    }

    /// cast the camera as a raw pointer
    pub fn as_ptr(&self) -> *const Camera3D {
        self as *const Camera3D
    }

    /// set the orientation vector of the camera
    ///
    /// # Arguments
    /// - `orientation` - The new orientation vector of the camera
    pub fn set_orientation_vector(&mut self, orientation: math::Vec3) -> &mut Self {
        let orientation = orientation.normalize();
        if orientation == math::vec3(0.0, 0.0, 1.0) {
            self.transform.set_rotation(math::Quat::IDENTITY);
            return self;
        }
        let rotation_axis = math::vec3(0.0, 0.0, 1.0).cross(orientation);
        let rotation_angle = math::vec3(0.0, 0.0, 1.0).dot(orientation).acos();
        let rotation_quat = math::Quat::from_axis_angle(rotation_axis, rotation_angle);
        self.transform.set_rotation(rotation_quat);

        self
    }

    /// get the orientation vector of the camera
    ///
    /// # Returns
    /// The orientation vector of the camera
    pub fn get_orientation_vector(&self) -> math::Vec3 {
        self.transform.get_forward_vector()
    }

    /// get the orientation angles of the camera
    ///
    /// # Returns
    /// The orientation angles of the camera
    pub fn get_orientation_angles(&self) -> math::Vec3 {
        //let default = math::vec3(0.0, 0.0, 1.0); //default orientation vector to compare to
        let pitch = (-self.transform.get_forward_vector().y).asin().to_degrees();
        let yaw = (self.transform.get_forward_vector().x)
            .atan2(self.transform.get_forward_vector().z)
            .to_degrees();
        let roll = 0.0;
        math::vec3(yaw, pitch, roll) //return the angles y is up
    }

    /// set the orientation angles of the camera
    ///
    /// # Arguments
    /// - `angles` - The new orientation angles of the camera
    pub fn set_orientation_angles(&mut self, angles: math::Vec3) {
        let yaw = angles.x.to_radians();
        let pitch = angles.y.to_radians();
        //let roll = math::radians(&math::vec1(angles.z)).x;

        let orientation = math::vec3(
            pitch.cos() * yaw.sin(),
            pitch.sin(),
            pitch.cos() * yaw.cos(),
        );
        self.set_orientation_vector(orientation);
    }

    /// get the view matrix of the camera
    ///
    /// # Returns
    /// The view matrix of the camera
    pub fn get_view_matrix(&self) -> math::Mat4 {
        //let world_position = parent_transform + self.transform;
        let world_position = self.transform.world_space();
        let target = world_position.position + self.transform.get_forward_vector();
        math::Mat4::look_at_rh(
            world_position.position,
            target,
            math::vec3(0.0, 1.0, 0.0), //up vector
        )
    }

    /// get the projection matrix of the camera
    ///
    /// # Returns
    /// The projection matrix of the camera
    pub fn get_projection_matrix(&self, aspect_ratio: f32) -> math::Mat4 {
        math::Mat4::perspective_rh(self.fov, aspect_ratio, self.near, self.far)
    }

    /// get the view projection matrix of the camera
    ///
    /// # Returns
    /// The view projection matrix of the camera
    pub fn get_vp_matrix(&self, aspect_ratio: f32) -> math::Mat4 {
        self.get_projection_matrix(aspect_ratio) * self.get_view_matrix()
    }

    pub fn get_buffer_data(&self, aspect_ratio: f32) -> Camera3DBufferData {
        let position = self.transform.world_space().position.extend(1.0).to_array();

        let view = self.get_view_matrix();
        let projection = self.get_projection_matrix(aspect_ratio);
        let vp = projection * view;

        Camera3DBufferData {
            position,
            view: view.to_cols_array_2d(),
            projection: projection.to_cols_array_2d(),
            vp: vp.to_cols_array_2d(),
        }
    }

    /// allows the mouse to rotate the camera in a first person way.
    ///
    /// uses camera.sensitivity to factor the look speed. add this function to the update callback to enable the camera to move with the mouse.
    pub fn free_look(&mut self, input: &InputManager, sensitivity: f32) {
        // Debug::print(&format!("{}", input.mouse_delta));

        let mouse_offset = input.mouse_delta;
        if mouse_offset != math::vec2(0.0, 0.0) {
            self.rotate_camera(math::vec3(mouse_offset.x, mouse_offset.y, 0.0), sensitivity);
        }
    }

    /// take input for the camera and implement basic free cam movement
    ///
    /// # Arguments
    /// - `input_manager` - The input manager to get input from
    /// - `delta_time` - The time between frames
    pub fn free_fly(
        &mut self,
        input_manager: &InputManager,
        delta_time: f32,
        sensitivity: f32,
        speed: f32,
    ) {
        let key = &input_manager.keys;

        let mut speed = speed * delta_time;

        let mut movement_offset = math::vec3(0.0, 0.0, 0.0);

        // the current right vector of the camera so that we know what direction to move diaganoly
        let right = self
            .transform
            .get_forward_vector()
            .cross(math::vec3(0.0, 1.0, 0.0))
            .normalize();

        // handle keys
        // if key.contains(&Key::LeftControl) {
        //     speed /= 5.0;
        // }
        if key.contains(&KeyCode::ShiftLeft) {
            speed *= 5.0;
        }
        if key.contains(&KeyCode::KeyW) {
            movement_offset += self.transform.get_forward_vector() * speed;
        }
        if key.contains(&KeyCode::KeyA) {
            movement_offset -= right * speed;
        }
        if key.contains(&KeyCode::KeyS) {
            movement_offset -= self.transform.get_forward_vector() * speed;
        }
        if key.contains(&KeyCode::KeyD) {
            movement_offset += right * speed;
        }
        if key.contains(&KeyCode::Space) {
            movement_offset += math::vec3(0.0, 1.0, 0.0) * speed;
        }
        if key.contains(&KeyCode::ControlLeft) {
            movement_offset -= math::vec3(0.0, 1.0, 0.0) * speed;
        }

        self.move_camera(movement_offset);

        let mouse_offset = input_manager.mouse_delta;
        if mouse_offset != math::vec2(0.0, 0.0) {
            self.rotate_camera(
                math::vec3(mouse_offset.x, mouse_offset.y, 0.0),
                sensitivity * delta_time,
            );
        }

        // handle mouse movement for rotation
        // if input_manager.mouse_buttons.contains(&MouseButton::Button3) {
        //     let mouse_offset: math::Vec2 =
        //         input_manager.mouse_position - input_manager.last_mouse_position;
        //     if mouse_offset != math::vec2(0.0, 0.0) {
        //         self.rotate_camera(
        //             math::vec3(mouse_offset.x, mouse_offset.y, 0.0),
        //             sensitivity * delta_time,
        //         );
        //     }
        // }
    }
}

impl Buildable for Camera3D {
    type Builder = Camera3DBuilder;
    fn builder() -> Self::Builder {
        Self::Builder {
            prototype: NodePrototype::default(),
            fov: FRAC_PI_4,
            far: 100.0,
            near: 0.1,
            active: true,
            priority: 0,
        }
    }
}

/// builder implementation for Camera3D
pub struct Camera3DBuilder {
    prototype: NodePrototype,
    fov: f32,
    near: f32,
    far: f32,
    active: bool,
    priority: i32,
}

impl Builder for Camera3DBuilder {
    type Node = Camera3D;
    fn prototype(&mut self) -> &mut NodePrototype {
        &mut self.prototype
    }

    fn build(self) -> Self::Node {
        Camera3D {
            transform: self.prototype.transform,
            events: self.prototype.events,
            children: self.prototype.children,
            far: self.far,
            near: self.near,
            fov: self.fov,
            priority: self.priority,
            is_active: self.active,
        }
    }
}

impl Camera3DBuilder {
    /// set the fov of the camera in radians
    pub fn fov(mut self, fov: f32) -> Self {
        self.fov = fov;
        self
    }

    /// far clipping plane of the camera. default: 100.0
    pub fn far_plane(mut self, far: f32) -> Self {
        self.far = far;
        self
    }

    /// near clipping plane of the camera. default: 0.1
    pub fn near_plane(mut self, near: f32) -> Self {
        self.near = near;
        self
    }

    /// whether the camera is active or not. default: true
    pub fn is_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// priority of the camera if more then 1 camera is active, default: 0
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// set the camera to look in the direction of a vector
    pub fn orientation_vector(mut self, mut orientation: math::Vec3) -> Self {
        orientation = orientation.normalize();
        if orientation == math::vec3(0.0, 0.0, 1.0) {
            self.prototype()
                .transform
                .set_rotation(math::Quat::IDENTITY);
            return self;
        }
        let rotation_axis = math::vec3(0.0, 0.0, 1.0).cross(orientation);
        let rotation_angle = math::vec3(0.0, 0.0, 1.0).dot(orientation).acos();
        let rotation_quat = math::Quat::from_axis_angle(rotation_axis, rotation_angle);
        self.prototype().transform.set_rotation(rotation_quat);

        self
    }
}

impl From<&Camera3D> for *const Camera3D {
    fn from(val: &Camera3D) -> Self {
        val as *const Camera3D
    }
}

impl From<&mut Camera3D> for *mut Camera3D {
    fn from(val: &mut Camera3D) -> Self {
        val as *mut Camera3D
    }
}
