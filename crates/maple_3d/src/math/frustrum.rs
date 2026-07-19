use glam::{Mat4, Vec3, Vec4};

use crate::math::AABB;

pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
}

impl Plane {
    pub fn normalize(&mut self) {
        let length = self.normal.length();
        self.normal /= length;
        self.distance /= length;
    }

    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        self.normal.dot(point) + self.distance
    }
}

pub struct Frustum {
    /// left, right, bottom, top, near, far
    pub planes: [Plane; 6],
}

impl Frustum {
    pub fn from_view_proj(vp: &Mat4) -> Self {
        // Grib-Hartmann method
        let m = vp.to_cols_array_2d();
        let mut planes = [
            // left: m[3] + m[0]
            Plane {
                normal: Vec3::new(m[0][3] + m[0][0], m[1][3] + m[1][0], m[2][3] + m[2][0]),
                distance: m[3][3] + m[3][0],
            },
            // right: m[3] - m[0]
            Plane {
                normal: Vec3::new(m[0][3] - m[0][0], m[1][3] - m[1][0], m[2][3] - m[2][0]),
                distance: m[3][3] - m[3][0],
            },
            // bottom: m[3] + m[1]
            Plane {
                normal: Vec3::new(m[0][3] + m[0][1], m[1][3] + m[1][1], m[2][3] + m[2][1]),
                distance: m[3][3] + m[3][1],
            },
            // top: m[3] - m[1]
            Plane {
                normal: Vec3::new(m[0][3] - m[0][1], m[1][3] - m[1][1], m[2][3] - m[2][1]),
                distance: m[3][3] - m[3][1],
            },
            // near: m[3] + m[2]
            Plane {
                normal: Vec3::new(m[0][3] + m[0][2], m[1][3] + m[1][2], m[2][3] + m[2][2]),
                distance: m[3][3] + m[3][2],
            },
            // far: m[3] - m[2]
            Plane {
                normal: Vec3::new(m[0][3] - m[0][2], m[1][3] - m[1][2], m[2][3] - m[2][2]),
                distance: m[3][3] - m[3][2],
            },
        ];

        // Normalize all planes
        for plane in &mut planes {
            plane.normalize();
        }

        Self { planes }
    }

    pub fn get_corners_from_view_proj(view_proj: &Mat4) -> [Vec3; 8] {
        let inverse_view_proj = view_proj.inverse();

        // NDC corners. Adjust the z range depending on your projection convention:
        // - Standard OpenGL-style NDC z: -1.0 (near) to 1.0 (far)
        // - Reverse-Z / wgpu/D3D-style NDC z: 1.0 (near) to 0.0 (far)
        let ndc_corners = [
            Vec3::new(-1.0, -1.0, 1.0), // near bottom-left
            Vec3::new(1.0, -1.0, 1.0),  // near bottom-right
            Vec3::new(-1.0, 1.0, 1.0),  // near top-left
            Vec3::new(1.0, 1.0, 1.0),   // near top-right
            Vec3::new(-1.0, -1.0, 0.0), // far bottom-left
            Vec3::new(1.0, -1.0, 0.0),  // far bottom-right
            Vec3::new(-1.0, 1.0, 0.0),  // far top-left
            Vec3::new(1.0, 1.0, 0.0),   // far top-right
        ];

        ndc_corners.map(|ndc| {
            let world = inverse_view_proj * Vec4::new(ndc.x, ndc.y, ndc.z, 1.0);
            world.truncate() / world.w
        })
    }

    pub fn intersects_aabb(&self, aabb: &AABB) -> bool {
        for plane in &self.planes {
            let p_vertex = Vec3::new(
                if plane.normal.x >= 0.0 {
                    aabb.max.x
                } else {
                    aabb.min.x
                },
                if plane.normal.y >= 0.0 {
                    aabb.max.y
                } else {
                    aabb.min.y
                },
                if plane.normal.z >= 0.0 {
                    aabb.max.z
                } else {
                    aabb.min.z
                },
            );

            if plane.distance_to_point(p_vertex) < 0.0 {
                return false;
            }
        }

        true
    }
}
