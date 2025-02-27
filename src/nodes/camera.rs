//! The Camera node is where the scene is rendered from.
//!
//! ## Usage
//! add a camera node the the scene and set the camera as the main camera in the game context and the engine will render the scene from the camera's perspective.
//!
//! ## Example
//! ```rust
//! use quaturn::game_context::nodes::camera::Camera3D;
//! use quaturn::game_context::GameContext;
//! use quaturn::game_context::node_manager::{Node, Scene};
//! use quaturn::Engine;
//! use quaturn::components::Event;
//!
//! use quaturn::glm;
//! let mut engine = Engine::init("Example", 800, 600);
//!
//! engine.context.nodes.add("Camera", Camera3D::new(
//!     glm::vec3(0.0, 0.0, 0.0),
//!     glm::vec3(0.0, 0.0, 1.0),
//!     45.0,
//!     800.0/600.0,
//!     0.1,
//!     100.0
//! )).on(Event::Ready, |camera, context| {
//!     // basic free cam movement (wasd, shift, space, ctrl, mouse)
//!     camera.take_input(&context.input, context.frame.time_delta.as_secs_f32());
//! });
//!
//! //engine.begin();
//! ```

extern crate nalgebra_glm as glm;
use egui_gl_glfw::glfw;

use glfw::Key;


use crate::components::{EventReceiver, NodeTransform};
use crate::context::{
    scene::{Node, Scene},
};

use super::{NodeBuilder};

/// A 2D camera that can be used to move around the screen. **Currently work in progress**.
pub struct Camera2D {
    height: f32,
    width: f32,
    position: glm::Vec2,
    zoom: f32,
}

impl Camera2D {
    /// the height and width of the camera are the height and width of the screen if changed will change the aspect ratio but not the size of the camera
    pub fn new(x: f32, y: f32, height: f32, width: f32) -> Camera2D {
        Camera2D {
            height,
            width,
            position: glm::vec2(x, y),
            zoom: 1.0,
        }
    }

    /// move the camera by an offset
    pub fn move_camera(&mut self, offset: glm::Vec2) {
        self.position += offset;
    }

    /// zoom the camera by a factor
    pub fn zoom_camera(&mut self, zoom: f32) {
        self.zoom += zoom;
    }

    /// set the height of the camera
    pub fn update_height(&mut self, height: f32) {
        self.height = height;
    }

    /// get the height of the camera
    pub fn get_height(&self) -> f32 {
        self.height
    }

    /// set the width of the camera
    pub fn update_width(&mut self, width: f32) {
        self.width = width;
    }

    /// get the width of the camera
    pub fn get_width(&self) -> f32 {
        self.width
    }

    /// get the zoom of the camera
    pub fn get_position(&self) -> glm::Vec2 {
        self.position
    }

    /// set the position of the camera
    pub fn set_position(&mut self, position: glm::Vec2) {
        self.position = position;
    }

    // This function returns the view matrix of the camera since if the camera is offset then the world is offset in the opposite direction
    // (should make a seemless coordinate system for the user even if the camera is technically at the origin)
    // mutliply this view matrix with the ortho and transform matrix to get the final MVP matrix

    /// get the view projection matrix of the camera
    pub fn get_vp_matrix(&self) -> glm::Mat4 {
        // the ortho matrix is the projection matrix
        let ortho = glm::ortho(
            -self.width / 2.0 * self.zoom,
            self.width / 2.0 * self.zoom,
            -self.height / 2.0 * self.zoom,
            self.height / 2.0 * self.zoom,
            -1.0,
            1.0,
        );
        // the translate matrix is the view matrix
        let translate = glm::translate(
            &glm::Mat4::identity(),
            &glm::vec3(-self.position.x, -self.position.y, 0.0),
        );
        ortho * translate
    }
}

// pub struct CameraTransform {
//     pub position: glm::Vec3,
//     pub orientation: glm::Vec3,
//     pub up: glm::Vec3,
// }

/// A 3D camera that can be use in a 3d environment.
#[derive(Clone)]
pub struct Camera3D {
    /// If the camera can be moved
    pub movement_enabled: bool,
    /// The sensitivity of the camera
    pub look_sensitivity: f32,
    /// The speed of the camera
    pub move_speed: f32,
    /// the NodeTransform of the camera (every node has this)
    pub transform: NodeTransform,
    /// the children of the camera (every node has this)
    pub children: Scene,
    /// events
    pub events: EventReceiver,
    /// the field of view of the camera in radians
    pub fov: f32,
    /// the aspect ratio of the camera
    aspect_ratio: f32,
    /// the near plane of the camera
    near: f32,
    /// the far plane of the camera
    far: f32,
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
    pub fn new(fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Camera3D {
        // println!("Camera created");
        // println!("Position: {:?}", position);
        // println!("Orientation: {:?}", orientation);
        // println!("FOV: {:?}", fov);
        // println!("Aspect Ratio: {:?}", aspect_ratio);
        // println!("Near: {:?}", near);
        // println!("Far: {:?}", far);

        // calculate the rotation quaternion from the orientation vector

        // println!("Rotation Quaternion: {:?}", rotation_quat);
        // println!("Rotation Axis: {:?}", rotation_axis);
        // println!("Rotation Angle: {:?}", rotation_angle);

        Camera3D {
            movement_enabled: true,
            look_sensitivity: 0.5,
            move_speed: 10.0,

            transform: NodeTransform::default(),
            children: Scene::new(),
            events: EventReceiver::new(),

            fov,
            aspect_ratio,
            near,
            far,
        }
    }

    pub fn set_orientation(&mut self, orientation: glm::Vec3) -> &mut Self {
        glm::normalize(&orientation);
        // if orientation default then reset quat
        if orientation == glm::vec3(0.0, 0.0, 1.0) {
            self.transform.set_rotation(glm::Quat::identity());
            return self;
        }

        let rotation_axis = glm::cross(&glm::vec3(0.0, 0.0, 1.0), &orientation);
        let rotation_angle = glm::dot(&glm::vec3(0.0, 0.0, 1.0), &orientation).acos();
        let rotation_quat = glm::quat_angle_axis(rotation_angle, &rotation_axis);

        self.transform.set_rotation(rotation_quat);

        self
    }

    /// offset the camera position
    ///
    /// # Arguments
    /// - `offset` - The offset to move the camera by a 3d vector
    pub fn move_camera(&mut self, offset: glm::Vec3) {
        //can be used to move the camera around the origin
        self.transform.position += offset;
    }

    /// rotate the camera while keeping the roll at 0
    ///
    /// # Arguments
    /// - `offset` - The offset to rotate the camera by a 3d vector
    /// - `sensitivity` - The sensitivity of the rotation
    pub fn rotate_camera(&mut self, offset: glm::Vec3, sensitivity: f32) {
        let max_pitch = glm::radians(&glm::vec1(89.90)).x; // prevent gimbal lock

        // Calculate pitch and yaw deltas
        let pitch_offset = offset.y * sensitivity;
        let yaw_offset = -offset.x * sensitivity;

        //println!("{}, {}", pitch_offset, yaw_offset);

        // Get the forward vector and calculate the current pitch
        let forward = self.transform.get_forward_vector().normalize();

        let current_pitch = glm::radians(&self.get_orientation_angles()).y;

        // Calculate the target pitch
        let target_pitch = glm::clamp_scalar(current_pitch + pitch_offset, -max_pitch, max_pitch);

        // Limit the pitch delta before applying it
        let clamped_pitch_offset = target_pitch - current_pitch;
        // println!("{}", clamped_pitch_offset); // This should be 0 when the current pitch is at the max_pitch but its not

        // Calculate the right vector
        let right = glm::normalize(&glm::cross(&glm::vec3(0.0, 1.0, 0.0), &forward)); // we cant use get_right_vector becuase it needs to be relative to the world up and forward not the camera up and forward

        // Create quaternions for pitch and yaw
        let pitch_quat = glm::quat_angle_axis(clamped_pitch_offset, &right);
        let yaw_quat = glm::quat_angle_axis(yaw_offset, &glm::vec3(0.0, 1.0, 0.0)); // rotate around world up

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
    pub fn set_position(&mut self, position: glm::Vec3) {
        self.transform.position = position;
    }

    /// get the position of the camera
    ///
    /// # Returns
    /// The position of the camera
    pub fn get_position(&self, parent_transform: NodeTransform) -> glm::Vec3 {
        (self.transform + parent_transform).position
    }

    pub fn as_ptr(&self) -> *const Camera3D {
        self as *const Camera3D
    }

    /// set the orientation vector of the camera
    ///
    /// # Arguments
    /// - `orientation` - The new orientation vector of the camera
    pub fn set_orientation_vector(&mut self, orientation: glm::Vec3) -> &mut Self {
        glm::normalize(&orientation);
        let rotation_axis = glm::cross(&glm::vec3(0.0, 0.0, 1.0), &orientation);
        let rotation_angle = glm::dot(&glm::vec3(0.0, 0.0, 1.0), &orientation).acos();
        let rotation_quat = glm::quat_angle_axis(rotation_angle, &rotation_axis);
        self.transform.set_rotation(rotation_quat);

        self
    }

    /// get the orientation vector of the camera
    ///
    /// # Returns
    /// The orientation vector of the camera
    pub fn get_orientation_vector(&self) -> glm::Vec3 {
        self.transform.get_forward_vector()
    }

    /// get the orientation angles of the camera
    ///
    /// # Returns
    /// The orientation angles of the camera
    pub fn get_orientation_angles(&self) -> glm::Vec3 {
        //let default = glm::vec3(0.0, 0.0, 1.0); //default orientation vector to compare to
        let pitch = (-self.transform.get_forward_vector().y).asin().to_degrees();
        let yaw = (self.transform.get_forward_vector().x)
            .atan2(self.transform.get_forward_vector().z)
            .to_degrees();
        let roll = 0.0;
        glm::vec3(yaw, pitch, roll) //return the angles y is up
    }

    /// set the orientation angles of the camera
    ///
    /// # Arguments
    /// - `angles` - The new orientation angles of the camera
    pub fn set_orientation_angles(&mut self, angles: glm::Vec3) {
        let yaw = glm::radians(&glm::vec1(angles.x)).x;
        let pitch = glm::radians(&glm::vec1(angles.y)).x;
        //let roll = glm::radians(&glm::vec1(angles.z)).x;

        let orientation = glm::vec3(
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
    pub fn get_view_matrix(&self, parent_transform: NodeTransform) -> glm::Mat4 {
        let world_position = self.transform + parent_transform;

        let target = world_position.position + self.transform.get_forward_vector();
        glm::look_at(
            &world_position.position,
            &target,
            &glm::vec3(0.0, 1.0, 0.0), //up vector
        )
    }

    /// get the projection matrix of the camera
    ///
    /// # Returns
    /// The projection matrix of the camera
    pub fn get_projection_matrix(&self) -> glm::Mat4 {
        glm::perspective(self.aspect_ratio, self.fov, self.near, self.far)
    }

    /// get the view projection matrix of the camera
    ///
    /// # Returns
    /// The view projection matrix of the camera
    pub fn get_vp_matrix(&self, parent_transform: NodeTransform) -> glm::Mat4 {
        self.get_projection_matrix() * self.get_view_matrix(parent_transform)
    }

    /// take input for the camera and implement basic free cam movement
    ///
    /// # Arguments
    /// - `input_manager` - The input manager to get input from
    /// - `delta_time` - The time between frames
    pub fn take_input(
        &mut self,
        input_manager: &crate::context::input_manager::InputManager,
        delta_time: f32,
    ) {
        if !self.movement_enabled {
            //println!("Input is disabled for the camera");
            return;
        }

        let key = &input_manager.keys;

        let mut speed = self.move_speed * delta_time;
        let sensitivity = self.look_sensitivity;

        let mut movement_offset = glm::vec3(0.0, 0.0, 0.0);

        // the current right vector of the camera so that we know what direction to move diaganoly
        let right = glm::normalize(&glm::cross(
            &self.transform.get_forward_vector(),
            &glm::vec3(0.0, 1.0, 0.0),
        ));

        // handle keys
        // if key.contains(&Key::LeftControl) {
        //     speed /= 5.0;
        // }
        if key.contains(&Key::LeftShift) {
            speed *= 5.0;
        }
        if key.contains(&Key::W) {
            movement_offset += self.transform.get_forward_vector() * speed;
        }
        if key.contains(&Key::A) {
            movement_offset -= right * speed;
        }
        if key.contains(&Key::S) {
            movement_offset -= self.transform.get_forward_vector() * speed;
        }
        if key.contains(&Key::D) {
            movement_offset += right * speed;
        }
        if key.contains(&Key::Space) {
            movement_offset += glm::vec3(0.0, 1.0, 0.0) * speed;
        }
        if key.contains(&Key::LeftControl) {
            movement_offset -= glm::vec3(0.0, 1.0, 0.0) * speed;
        }

        self.move_camera(movement_offset);

        let mouse_offset = input_manager.mouse_delta;
        if mouse_offset != glm::vec2(0.0, 0.0) {
            self.rotate_camera(
                glm::vec3(mouse_offset.x, mouse_offset.y, 0.0),
                sensitivity * delta_time,
            );
        }

        // handle mouse movement for rotation
        // if input_manager.mouse_buttons.contains(&MouseButton::Button3) {
        //     let mouse_offset: glm::Vec2 =
        //         input_manager.mouse_position - input_manager.last_mouse_position;
        //     if mouse_offset != glm::vec2(0.0, 0.0) {
        //         self.rotate_camera(
        //             glm::vec3(mouse_offset.x, mouse_offset.y, 0.0),
        //             sensitivity * delta_time,
        //         );
        //     }
        // }
    }
}

pub trait Camera3DBuilder {
    fn set_orientation_vector(&mut self, orientation: glm::Vec3) -> &mut Self;
}

impl Camera3DBuilder for NodeBuilder<Camera3D> {
    fn set_orientation_vector(&mut self, mut orientation: nalgebra_glm::Vec3) -> &mut Self {
        orientation = orientation.normalize();
        let rotation_axis = glm::cross(&glm::vec3(0.0, 0.0, 1.0), &orientation);
        let rotation_angle = glm::dot(&glm::vec3(0.0, 0.0, 1.0), &orientation).acos();
        let rotation_quat = glm::quat_angle_axis(rotation_angle, &rotation_axis);
        self.transform.set_rotation(rotation_quat);

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
