use std::f32::consts::PI;

use bytemuck::Zeroable;
use glam::Vec2;
use maple_engine::asset::{Asset, AssetLibrary, IntoAsset, LoadErr};

use crate::{assets::mesh::Mesh3D, math::Vertex};

/// represents a capsule for making a capsule shaped [`Mesh3D`]
pub struct Capsule {
    /// height of the capsules inner cylinder
    ///
    /// default: 1.0
    pub height: f32,
    /// radius of inner cylinder
    ///
    /// default: 0.5
    pub radius: f32,
    /// how many latitude segments there are
    ///
    /// default: 16
    pub latitudes: u32,
    /// how many longitude segments there are
    ///
    /// default: 32
    pub longitudes: u32,
    /// inner rings of the cylinder
    ///
    /// default: 0
    pub rings: u32,
}

impl Default for Capsule {
    fn default() -> Self {
        Self {
            height: 1.0,
            radius: 0.5,
            latitudes: 16,
            longitudes: 32,
            rings: 0,
        }
    }
}

impl Capsule {
    /// height of the capsules inner cylinder
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// radius of inner cylinder
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// how many latitude segments there are
    pub fn latitudes(mut self, latitudes: u32) -> Self {
        self.latitudes = latitudes;
        self
    }

    /// how many longitude segments there are
    pub fn longitudes(mut self, longitudes: u32) -> Self {
        self.longitudes = longitudes;
        self
    }

    /// inner rings of the cylinder
    pub fn rings(mut self, rings: u32) -> Self {
        self.rings = rings;
        self
    }
}

impl IntoAsset<Mesh3D> for Capsule {
    async fn into_asset(
        self,
        loader: &<Mesh3D as Asset>::Loader,
        _library: &AssetLibrary,
    ) -> Result<Mesh3D, LoadErr> {
        // code from: https://behreajj.medium.com/making-a-capsule-mesh-via-script-in-five-3d-environments-c2214abf02db
        let latitudes = if self.latitudes % 2 != 0 {
            self.latitudes + 1
        } else {
            self.latitudes
        };

        let half_lats = latitudes / 2;

        // --- Vertex-space offsets ---
        let offset_north_hemi = self.longitudes;
        let offset_north_equator = offset_north_hemi + (self.longitudes + 1) * (half_lats - 1);
        let offset_cylinder = offset_north_equator + (self.longitudes + 1);
        let offset_south_equator = if self.rings > 0 {
            offset_cylinder + (self.longitudes + 1) * (self.rings + 1)
        } else {
            offset_cylinder
        };
        let offset_south_hemi = offset_south_equator + (self.longitudes + 1);
        let offset_south_polar = offset_south_hemi + (self.longitudes + 1) * (half_lats - 2);
        let offset_south_cap = offset_south_polar + (self.longitudes + 1);

        let vertex_count = offset_south_cap + self.longitudes;

        let mut vertices: Vec<Vertex> = vec![Vertex::zeroed(); vertex_count as usize];

        let to_theta = 2.0 * PI / self.longitudes as f32;
        let to_phi = PI / latitudes as f32;
        let to_tex_horizontal = 1.0 / self.longitudes as f32;
        let to_tex_vertical = 1.0 / half_lats as f32;

        let aspect_ratio = 1.0 / 3.0;
        let aspect_north = 1.0 / aspect_ratio;
        let aspect_south = aspect_ratio;

        let mut theta_cartesian: Vec<Vec2> = vec![Vec2::ZERO; self.longitudes as usize];
        let mut rho_theta_cartesian: Vec<Vec2> = vec![Vec2::ZERO; self.longitudes as usize];
        let mut tex_cache: Vec<f32> = vec![0.0; self.longitudes as usize + 1];

        // polar
        for lon in 0..self.longitudes {
            let lf = lon as f32;
            let tex_polar = 1.0 - ((lf + 0.5) * to_tex_horizontal);
            let cos_theta = f32::cos(lf * to_theta);
            let sin_theta = f32::sin(lf * to_theta);

            theta_cartesian[lon as usize] = Vec2::new(cos_theta, sin_theta);
            rho_theta_cartesian[lon as usize] = self.radius * Vec2::new(cos_theta, sin_theta);

            // north
            vertices[lon as usize] = Vertex {
                position: [0.0, self.height / 2.0 + self.radius, 0.0],
                normal: [0.0, 1.0, 0.0],
                tex_uv: [tex_polar, 0.0],
                ..Zeroable::zeroed()
            };

            // south
            let i = offset_south_cap + lon;
            vertices[i as usize] = Vertex {
                position: [0.0, -(self.height / 2.0 + self.radius), 0.0],
                normal: [0.0, -1.0, 0.0],
                tex_uv: [tex_polar, 0.0],
                ..Zeroable::zeroed()
            };
        }

        // equatorial
        for lon in 0..=self.longitudes {
            let tex = 1.0 - lon as f32 * to_tex_horizontal;
            tex_cache[lon as usize] = tex;

            let ll = lon % self.longitudes;
            let tc = theta_cartesian[ll as usize];
            let rtc = rho_theta_cartesian[ll as usize];

            // north equator
            let idxn = offset_north_equator + lon;
            vertices[idxn as usize] = Vertex {
                position: [rtc.x, self.height / 2.0, -rtc.y],
                normal: [tc.x, 0.0, -tc.y],
                tex_uv: [tex, aspect_north],
                ..Zeroable::zeroed()
            };

            // south equator
            let idxs = offset_south_equator + lon;
            vertices[idxs as usize] = Vertex {
                position: [rtc.x, -self.height / 2.0, -rtc.y],
                normal: [tc.x, 0.0, -tc.y],
                tex_uv: [tex, aspect_south],
                ..Zeroable::zeroed()
            };
        }

        // hemisphere
        for lat in 0..half_lats - 1 {
            let phi = (lat as f32 + 1.0) * to_phi;
            let cos_phi_south = f32::cos(phi);
            let sin_phi_south = f32::sin(phi);
            let cos_phi_north = sin_phi_south;
            let sin_phi_north = -cos_phi_south;

            let rho_cos_phi_north = self.radius * cos_phi_north;
            let rho_sin_phi_north = self.radius * sin_phi_north;
            let offset_north = self.height / 2.0 - rho_sin_phi_north;
            let rho_cos_phi_south = self.radius * cos_phi_south;
            let rho_sin_phi_south = self.radius * sin_phi_south;
            let offset_south = -self.height / 2.0 - rho_sin_phi_south;

            let tex_fac = (lat as f32 + 1.0) * to_tex_vertical;
            let cmpl_tex_fac = 1.0 - tex_fac;
            let tex_north = cmpl_tex_fac + aspect_north * tex_fac;
            let tex_south = cmpl_tex_fac * aspect_south;

            let current_lat_north = offset_north_hemi + (lat * (self.longitudes + 1));
            let current_lat_south = offset_south_hemi + (lat * (self.longitudes + 1));

            for lon in 0..=self.longitudes {
                let tex = tex_cache[lon as usize];
                let tc = theta_cartesian[(lon % self.longitudes) as usize];

                // north hemisphere
                let idxn = current_lat_north + lon;
                vertices[idxn as usize] = Vertex {
                    position: [
                        rho_cos_phi_north * tc.x,
                        offset_north,
                        -rho_cos_phi_north * tc.y,
                    ],
                    normal: [cos_phi_north * tc.x, -sin_phi_north, -cos_phi_north * tc.y],
                    tex_uv: [tex, tex_north],
                    ..Zeroable::zeroed()
                };

                // south hemisphere
                let idxs = current_lat_south + lon;
                vertices[idxs as usize] = Vertex {
                    position: [
                        rho_cos_phi_south * tc.x,
                        offset_south,
                        -rho_cos_phi_south * tc.y,
                    ],
                    normal: [cos_phi_south * tc.x, -sin_phi_south, -cos_phi_south * tc.y],
                    tex_uv: [tex, tex_south],
                    ..Zeroable::zeroed()
                };
            }
        }

        // cylinder
        if self.rings > 0 {
            let to_fac = 1.0 / (self.rings + 1) as f32;
            let mut index = offset_cylinder;

            for h in 0..=self.rings {
                let fac = h as f32 * to_fac;
                let cmpl_fac = 1.0 - fac;
                let tex = cmpl_fac * aspect_north + fac * aspect_south;
                let z = (self.height / 2.0) - self.height * fac;

                for lon in 0..=self.longitudes {
                    let tc = theta_cartesian[(lon % self.longitudes) as usize];
                    let rtc = rho_theta_cartesian[(lon % self.longitudes) as usize];
                    let stex = tex_cache[lon as usize];
                    vertices[index as usize] = Vertex {
                        position: [rtc.x, z, -rtc.y],
                        normal: [tc.x, 0.0, -tc.y],
                        tex_uv: [stex, tex],
                        ..Zeroable::zeroed()
                    };
                    index += 1;
                }
            }
        }

        let long3 = self.longitudes * 3;
        let long6 = self.longitudes * 6;
        let hemi_long = (half_lats - 1) * long6;

        let idx_offset_north_hemi = long3;
        let idx_offset_cylinder = idx_offset_north_hemi + hemi_long;
        let idx_offset_south_hemi = idx_offset_cylinder + (self.rings + 2) * long6;
        let idx_offset_south_cap = idx_offset_south_hemi + hemi_long;

        let tri_count = idx_offset_south_cap + long3;

        let mut indices: Vec<u32> = vec![0u32; tri_count as usize];

        // polar caps
        {
            let mut k = 0;
            let mut m = idx_offset_south_cap;
            for i in 0..self.longitudes {
                // north
                indices[k as usize] = i;
                indices[(k + 1) as usize] = offset_north_hemi + i;
                indices[(k + 2) as usize] = offset_north_hemi + i + 1;
                // south
                indices[m as usize] = offset_south_cap + i;
                indices[(m + 1) as usize] = offset_south_polar + i + 1;
                indices[(m + 2) as usize] = offset_south_polar + i;

                k += 3;
                m += 3;
            }
        }

        // hemispheres
        {
            let mut k = idx_offset_north_hemi;
            let mut m = idx_offset_south_hemi;
            for i in 0..(half_lats - 1) {
                let current_lat_north = offset_north_hemi + (i * (self.longitudes + 1));
                let next_lat_north = current_lat_north + (self.longitudes + 1);
                let current_lat_south = offset_south_equator + (i * (self.longitudes + 1));
                let next_lat_south = current_lat_south + (self.longitudes + 1);

                for j in 0..self.longitudes {
                    // north
                    let n00 = current_lat_north + j;
                    let n01 = next_lat_north + j;
                    let n11 = next_lat_north + j + 1;
                    let n10 = current_lat_north + j + 1;

                    indices[k as usize] = n00;
                    indices[k as usize + 1] = n11;
                    indices[k as usize + 2] = n10;

                    indices[k as usize + 3] = n00;
                    indices[k as usize + 4] = n01;
                    indices[k as usize + 5] = n11;

                    // south
                    let s00 = current_lat_south + j;
                    let s01 = next_lat_south + j;
                    let s11 = next_lat_south + j + 1;
                    let s10 = current_lat_south + j + 1;

                    indices[m as usize] = s00;
                    indices[m as usize + 1] = s11;
                    indices[m as usize + 2] = s10;

                    indices[m as usize + 3] = s00;
                    indices[m as usize + 4] = s01;
                    indices[m as usize + 5] = s11;

                    k += 6;
                    m += 6;
                }
            }
        }

        // cylinder
        {
            let mut k = idx_offset_cylinder;
            for i in 0..=(self.rings + 1) {
                let current_lat = offset_north_equator + i * (self.longitudes + 1);
                let next_lat = current_lat + (self.longitudes + 1);

                for j in 0..self.longitudes {
                    let cy00 = current_lat + j;
                    let cy01 = next_lat + j;
                    let cy11 = next_lat + j + 1;
                    let cy10 = current_lat + j + 1;

                    indices[k as usize] = cy00;
                    indices[k as usize + 1] = cy11;
                    indices[k as usize + 2] = cy10;
                    indices[k as usize + 3] = cy00;
                    indices[k as usize + 4] = cy01;
                    indices[k as usize + 5] = cy11;

                    k += 6;
                }
            }
        }

        let mesh = loader.create_mesh(&mut vertices, &indices);
        Ok(mesh)
    }
}
