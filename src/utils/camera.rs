extern crate nalgebra_glm as glm;

pub struct Camera2D {
    position: glm::Vec2,
}

impl Camera2D {
    pub fn new() -> Camera2D {
        Camera2D {
            position: glm::vec2(0.0, 0.0),
        }
    }

    pub fn move_camera(&mut self, offset: glm::Vec2) {
        self.position += offset;
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
    pub fn get_view_matrix(&self) -> glm::Mat4 {
        glm::translate(
            &glm::Mat4::identity(),
            &glm::vec3(-self.position.x, -self.position.y, 0.0),
        )
    }
}
