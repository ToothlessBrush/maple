use glam::{Mat4, Vec3};
use maple_renderer::types::Vertex;

#[derive(Clone, Copy, Debug)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    /// create an AABB bounding box from a list of vertices
    pub fn from_vertices(positions: &[Vertex]) -> Self {
        let Some(first) = positions.first() else {
            return Self {
                min: Vec3::ZERO,
                max: Vec3::ZERO,
            };
        };

        let mut min_x = first.position[0];
        let mut min_y = first.position[1];
        let mut min_z = first.position[2];
        let mut max_x = first.position[0];
        let mut max_y = first.position[1];
        let mut max_z = first.position[2];

        for v in positions.iter().skip(1) {
            min_x = min_x.min(v.position[0]);
            min_y = min_y.min(v.position[1]);
            min_z = min_z.min(v.position[2]);
            max_x = max_x.max(v.position[0]);
            max_y = max_y.max(v.position[1]);
            max_z = max_z.max(v.position[2]);
        }

        Self {
            min: Vec3::new(min_x, min_y, min_z),
            max: Vec3::new(max_x, max_y, max_z),
        }
    }

    pub fn from_positions(positions: &[[f32; 3]]) -> Self {
        let Some(first) = positions.first() else {
            return Self {
                min: Vec3::ZERO,
                max: Vec3::ZERO,
            };
        };

        let mut min_x = first[0];
        let mut min_y = first[1];
        let mut min_z = first[2];
        let mut max_x = first[0];
        let mut max_y = first[1];
        let mut max_z = first[2];

        for v in positions.iter().skip(1) {
            min_x = min_x.min(v[0]);
            min_y = min_y.min(v[1]);
            min_z = min_z.min(v[2]);
            max_x = max_x.max(v[0]);
            max_y = max_y.max(v[1]);
            max_z = max_z.max(v[2]);
        }

        Self {
            min: Vec3::new(min_x, min_y, min_z),
            max: Vec3::new(max_x, max_y, max_z),
        }
    }

    /// get the bounding box corners
    pub fn corners(&self) -> [Vec3; 8] {
        [
            Vec3::new(self.min.x, self.min.y, self.min.z),
            Vec3::new(self.max.x, self.min.y, self.min.z),
            Vec3::new(self.min.x, self.max.y, self.min.z),
            Vec3::new(self.max.x, self.max.y, self.min.z),
            Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.min.y, self.max.z),
            Vec3::new(self.min.x, self.max.y, self.max.z),
            Vec3::new(self.max.x, self.max.y, self.max.z),
        ]
    }

    pub fn transform(&self, model: &Mat4) -> Self {
        let corners = self.corners();
        let transformed: [[f32; 3]; 8] = corners.map(|corner| {
            let v4 = model.mul_vec4(corner.extend(1.0));
            (v4.truncate() / v4.w).into()
        });

        Self::from_positions(&transformed)
    }
}
