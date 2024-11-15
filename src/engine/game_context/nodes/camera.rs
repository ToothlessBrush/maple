extern crate nalgebra_glm as glm;
use egui_gl_glfw::glfw;

use glfw::Key;

use crate::engine::game_context::{node_manager::Node, GameContext};

pub struct Camera2D {
    height: f32,
    width: f32,
    position: glm::Vec2,
    zoom: f32,
}

impl Camera2D {
    //the height and width of the camera are the height and width of the screen if changed will change the aspect ratio but not the size of the camera
    pub fn new(x: f32, y: f32, height: f32, width: f32) -> Camera2D {
        Camera2D {
            height,
            width,
            position: glm::vec2(x, y),
            zoom: 1.0,
        }
    }

    pub fn move_camera(&mut self, offset: glm::Vec2) {
        self.position += offset;
    }

    pub fn zoom_camera(&mut self, zoom: f32) {
        self.zoom += zoom;
    }

    pub fn update_height(&mut self, height: f32) {
        self.height = height;
    }

    pub fn get_height(&self) -> f32 {
        self.height
    }

    pub fn update_width(&mut self, width: f32) {
        self.width = width;
    }

    pub fn get_width(&self) -> f32 {
        self.width
    }

    pub fn get_position(&self) -> glm::Vec2 {
        self.position
    }

    pub fn set_position(&mut self, position: glm::Vec2) {
        self.position = position;
    }

    // This function returns the view matrix of the camera since if the camera is offset then the world is offset in the opposite direction
    // (should make a seemless coordinate system for the user even if the camera is technically at the origin)
    // mutliply this view matrix with the ortho and transform matrix to get the final MVP matrix
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

pub struct CameraTransform {
    pub position: glm::Vec3,
    pub orientation: glm::Vec3,
    pub up: glm::Vec3,
}

pub struct Camera3D {
    pub movement_enabled: bool,
    pub look_sensitivity: f32,
    pub move_speed: f32,

    transform: CameraTransform,
    pub fov: f32,
    aspect_ratio: f32,
    near: f32,
    far: f32,

    ready_callback: Option<Box<dyn FnMut(&mut Self)>>,
    behavior_callback: Option<Box<dyn FnMut(&mut Self, &mut GameContext)>>,
}

impl Node for Camera3D {
    type Transform = CameraTransform;

    fn get_model_matrix(&self) -> glm::Mat4 {
        glm::identity()
    }

    fn get_transform(&self) -> &Self::Transform {
        &self.transform
    }

    fn define_ready<F>(&mut self, ready_function: F) -> &mut Self
    where
        F: 'static + FnMut(&mut Self),
    {
        self.ready_callback = Some(Box::new(ready_function));
        self
    }

    fn define_behavior<F>(&mut self, behavior_function: F) -> &mut Self
    where
        F: 'static + FnMut(&mut Self, &mut GameContext),
    {
        self.behavior_callback = Some(Box::new(behavior_function));
        self
    }

    //if the model has a ready function then call it
    fn ready(&mut self) {
        if let Some(mut callback) = self.ready_callback.take() {
            callback(self);
            self.ready_callback = Some(callback);
        }
    }

    //if the model has a behavior function then call it
    fn behavior(&mut self, context: &mut GameContext) {
        if let Some(mut callback) = self.behavior_callback.take() {
            callback(self, context);
            self.behavior_callback = Some(callback);
        }
    }
}

impl Camera3D {
    pub fn new(
        position: glm::Vec3,
        orientation: glm::Vec3,
        fov: f32,
        aspect_ratio: f32,
        near: f32,
        far: f32,
    ) -> Camera3D {
        println!("Camera created");
        println!("Position: {:?}", position);
        println!("Orientation: {:?}", orientation);
        println!("FOV: {:?}", fov);
        println!("Aspect Ratio: {:?}", aspect_ratio);
        println!("Near: {:?}", near);
        println!("Far: {:?}", far);

        Camera3D {
            movement_enabled: true,
            look_sensitivity: 0.5,
            move_speed: 10.0,

            transform: CameraTransform {
                position,
                orientation,
                up: glm::vec3(0.0, 1.0, 0.0),
            },
            fov,
            aspect_ratio,
            near,
            far,

            ready_callback: None,
            behavior_callback: None,
        }
    }

    pub fn move_camera(&mut self, offset: glm::Vec3) {
        //can be used to move the camera around the origin
        self.transform.position += offset;
    }

    pub fn rotate_camera(&mut self, offset: glm::Vec3, sensitvity: f32) {
        //vec3 contains x y z of the rotation
        //need to implement a way to rotate the camera while keeping the orientation vector normalized at 1
        //this will allow the camera to rotate around the origin
        let max_pitch = glm::radians(&glm::vec1(89.0)).x;

        let mut pitch = offset.y * sensitvity * -1.0;
        let yaw = offset.x * sensitvity * -1.0;
        //let roll = offset.z * sensitvity;

        let current_pitch = (self.transform.orientation.y).asin();

        pitch = glm::clamp_scalar(pitch + current_pitch, -max_pitch, max_pitch) - current_pitch; //if the pitch is greater than the max pitch then set it to the max pitch and subtract the current pitch to get the difference

        let right = glm::normalize(&glm::cross(&self.transform.orientation, &self.transform.up));

        let yaw_quat: glm::Quat = glm::quat_angle_axis(yaw, &self.transform.up);
        let pitch_quat: glm::Quat = glm::quat_angle_axis(pitch, &right);

        let combined_quat = yaw_quat * pitch_quat;

        let combined_quat = combined_quat.normalize();
        self.transform.orientation =
            glm::quat_rotate_vec3(&combined_quat, &self.transform.orientation);
    }

    pub fn set_position(&mut self, position: glm::Vec3) {
        self.transform.position = position;
    }

    pub fn get_position(&self) -> glm::Vec3 {
        self.transform.position
    }

    pub fn set_orientation(&mut self, orientation: glm::Vec3) {
        self.transform.orientation = orientation.normalize();
    }

    pub fn get_orientation(&self) -> glm::Vec3 {
        self.transform.orientation
    }

    pub fn get_orientation_angles(&self) -> glm::Vec3 {
        //let default = glm::vec3(0.0, 0.0, 1.0); //default orientation vector to compare to
        let pitch = (-self.transform.orientation.y).asin().to_degrees();
        let yaw = (self.transform.orientation.x)
            .atan2(self.transform.orientation.z)
            .to_degrees();
        let roll = 0.0;
        glm::vec3(yaw, pitch, roll) //return the angles y is up
    }

    pub fn set_orientation_angles(&mut self, angles: glm::Vec3) {
        let yaw_quat: glm::Quat =
            glm::quat_angle_axis(angles.x.to_radians(), &glm::vec3(0.0, 1.0, 0.0));
        let pitch_quat: glm::Quat =
            glm::quat_angle_axis(angles.y.to_radians(), &glm::vec3(1.0, 0.0, 0.0));
        let roll_quat: glm::Quat =
            glm::quat_angle_axis(angles.z.to_radians(), &glm::vec3(0.0, 0.0, 1.0));

        let combined_quat = yaw_quat * pitch_quat * roll_quat;

        self.transform.orientation =
            glm::quat_rotate_vec3(&combined_quat, &glm::vec3(0.0, 0.0, 1.0));
    }

    pub fn get_view_matrix(&self) -> glm::Mat4 {
        let target = self.transform.position + self.transform.orientation;
        glm::look_at(&self.transform.position, &target, &self.transform.up)
    }

    pub fn get_projection_matrix(&self) -> glm::Mat4 {
        glm::perspective(self.aspect_ratio, self.fov, self.near, self.far)
    }

    pub fn get_vp_matrix(&self) -> glm::Mat4 {
        self.get_projection_matrix() * self.get_view_matrix()
    }

    pub fn take_input(
        &mut self,
        input_manager: &crate::engine::game_context::input_manager::InputManager,
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
        let right = glm::normalize(&glm::cross(&self.transform.orientation, &self.transform.up));

        // handle keys
        // if key.contains(&Key::LeftControl) {
        //     speed /= 5.0;
        // }
        if key.contains(&Key::LeftShift) {
            speed *= 5.0;
        }
        if key.contains(&Key::W) {
            movement_offset += self.transform.orientation * speed;
        }
        if key.contains(&Key::A) {
            movement_offset -= right * speed;
        }
        if key.contains(&Key::S) {
            movement_offset -= self.transform.orientation * speed;
        }
        if key.contains(&Key::D) {
            movement_offset += right * speed;
        }
        if key.contains(&Key::Space) {
            movement_offset += self.transform.up * speed;
        }
        if key.contains(&Key::LeftControl) {
            movement_offset -= self.transform.up * speed;
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

    pub fn define_ready<F>(&mut self, ready_function: F) -> &mut Camera3D
    where
        F: FnMut(&mut Camera3D) + 'static,
    {
        self.ready_callback = Some(Box::new(ready_function));
        self
    }

    pub fn define_behavior<F>(&mut self, behavior_function: F) -> &mut Camera3D
    where
        F: FnMut(&mut Camera3D, &mut GameContext) + 'static,
    {
        self.behavior_callback = Some(Box::new(behavior_function));
        self
    }

    //if the model has a ready function then call it
    pub fn ready(&mut self) {
        if let Some(mut callback) = self.ready_callback.take() {
            callback(self);
            self.ready_callback = Some(callback);
        }
    }

    //if the model has a behavior function then call it
    pub fn behavior(&mut self, context: &mut GameContext) {
        if let Some(mut callback) = self.behavior_callback.take() {
            callback(self, context);
            self.behavior_callback = Some(callback);
        }
    }
}
