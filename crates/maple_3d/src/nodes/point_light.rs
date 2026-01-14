//! Point lights like their name emit light from a point in space
//!
//! This module provides a point light node that can be add to a scene.
//! each point light has a configurable position, color, and intensity.

const MAX_LIGHTS: usize = 100;

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3, Vec4};
use maple_engine::{
    Buildable, Builder, Node,
    nodes::node_builder::NodePrototype,
    prelude::{NodeTransform},
    utils::Color,
};

/// used to pass data to the shader buffer
///
/// the data on the gpu follows this format in this order:
/// ```c
/// struct PointLight {
///     vec4 color;
///     vec4 pos;
///     float intensity;
///     int shadowIndex;
///     float far_plane;
///     int _padding;
/// };
/// ```
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct PointLightBufferData {
    color: [f32; 4],
    position: [f32; 4],
    intensity: f32,
    shadow_index: i32,
    far_plane: f32,
    bias: f32, //ssbo is 16 byte aligned
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct PointLightBuffer {
    pub length: i32,
    _padding: [i32; 3],
    pub data: [PointLightBufferData; MAX_LIGHTS],
}

impl PointLightBuffer {
    pub fn from_lights(lights: &[PointLightBufferData]) -> Self {
        let mut buffer = PointLightBuffer {
            length: lights.len().min(MAX_LIGHTS) as i32,
            _padding: [0; 3],
            data: [PointLightBufferData::default(); MAX_LIGHTS],
        };

        let copy_count = lights.len().min(MAX_LIGHTS);
        buffer.data[..copy_count].copy_from_slice(&lights[..copy_count]);

        buffer
    }
}

/// point lights nodes represent point lights in the Scene
///
/// point lights are lights that are cast from a single point. light is calculated by getting the
/// distance and direction to the point light position.
pub struct PointLight {
    /// transform component for point light
    pub transform: NodeTransform,

    /// event receiver component

    /// the light intensity (simply factors the color by a scale)
    intensity: f32,

    /// the light color default is White
    pub color: Vec4,

    projection: Mat4,

    //shadow_map: DepthCubeMap,
    far_plane: f32,

    near_plane: f32,

    pub bias: f32,
}

impl Node for PointLight {
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }

}

impl Default for PointLight {
    fn default() -> Self {
        Self::new()
    }
}

impl PointLight {
    /// create a point light.
    pub fn new() -> PointLight {
        let transform = NodeTransform::default();

        let shadow_proj = Mat4::perspective_rh(90.0_f32.to_radians(), 1.0, 0.1, 10.0);

        PointLight {
            intensity: 1.0,
            projection: shadow_proj,
            near_plane: 0.01,
            far_plane: 10.0,
            transform,
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            bias: 0.0001,
        }
    }

    /// returns the formatted buffer data
    pub fn get_buffered_data(&self, index: usize) -> PointLightBufferData {
        let position: [f32; 3] = self.transform.world_space().position.into();
        let sized_positon = [position[0], position[1], position[2], 0.0];

        PointLightBufferData {
            color: self.color.into(),
            position: sized_positon,
            intensity: self.intensity,
            shadow_index: index as i32,
            far_plane: self.far_plane,
            bias: self.bias,
        }
    }

    /// get the nodes intensity
    pub fn get_intensity(&self) -> f32 {
        self.intensity
    }

    /// get the nodes intensity
    pub fn get_intensity_mut(&mut self) -> &mut f32 {
        &mut self.intensity
    }

    /// how strong the light is.
    ///
    /// this will also update the far plane for shadows since more intense lights go further
    pub fn set_intensity(&mut self, intensity: f32) {
        self.intensity = intensity;
        self.far_plane = Self::calculate_far_plane(intensity, 0.01);
        self.update_shadow_projection();
    }

    /// calculate the far_plane for a given intensity so that the shadow cutoff is at a light level
    /// thrshhold
    ///
    /// to save resources we only want to render shadows as far as the light is visible so we can
    /// cut it off at a given threshold
    pub fn calculate_far_plane(intensity: f32, threshold: f32) -> f32 {
        (intensity / threshold).sqrt()
    }

    fn update_shadow_projection(&mut self) {
        let shadow_proj =
            Mat4::perspective_rh(90.0_f32.to_radians(), 1.0, self.near_plane, self.far_plane);

        self.projection = shadow_proj;
    }

    pub fn get_shadow_transformations(&self) -> [Mat4; 6] {
        let transform = self.transform.world_space();
        let pos = transform.position;
        let shadow_proj = self.projection;

        [
            shadow_proj * Mat4::look_at_rh(pos, pos + Vec3::X, Vec3::NEG_Y),
            shadow_proj * Mat4::look_at_rh(pos, pos + Vec3::NEG_X, Vec3::NEG_Y),
            // for some reason the Y's are flipped so I flip them here
            shadow_proj * Mat4::look_at_rh(pos, pos + Vec3::NEG_Y, Vec3::NEG_Z),
            shadow_proj * Mat4::look_at_rh(pos, pos + Vec3::Y, Vec3::Z),
            shadow_proj * Mat4::look_at_rh(pos, pos + Vec3::Z, Vec3::NEG_Y),
            shadow_proj * Mat4::look_at_rh(pos, pos + Vec3::NEG_Z, Vec3::NEG_Y),
        ]
    }

    /// set the light color
    pub fn set_color(&mut self, color: impl Into<Vec4>) -> &mut Self {
        let color = color.into();
        self.color = color;
        self
    }

    /// get the light color
    pub fn get_color_mut(&mut self) -> &mut Vec4 {
        &mut self.color
    }
}

impl Buildable for PointLight {
    type Builder = PointLightBuilder;
    fn builder() -> Self::Builder {
        Self::Builder {
            prototype: NodePrototype::default(),
            intensity: 1.0,
            color: Color::WHITE.into(),
            near_plane: 0.1,
            bias: 0.0001,
        }
    }
}

/// point light specific builder
pub struct PointLightBuilder {
    prototype: NodePrototype,
    intensity: f32,
    color: Vec4,
    near_plane: f32,
    bias: f32,
}

impl Builder for PointLightBuilder {
    type Node = PointLight;
    fn prototype(&mut self) -> &mut NodePrototype {
        &mut self.prototype
    }

    fn build(self) -> Self::Node {
        let far_plane = PointLight::calculate_far_plane(self.intensity, 0.01);
        let mut light = Self::Node {
            transform: self.prototype.transform,
            color: self.color,
            intensity: self.intensity,
            near_plane: self.near_plane,
            far_plane,
            projection: Mat4::default(),
            bias: self.bias,
        };

        light.update_shadow_projection();
        light
    }
}

impl PointLightBuilder {
    /// set the intensity of the light
    pub fn intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        self
    }

    /// set the color of the light
    pub fn color(mut self, color: impl Into<Vec4>) -> Self {
        self.color = color.into();
        self
    }

    /// near clipping plane of the light shadow projections
    pub fn near_plane(mut self, near_plane: f32) -> Self {
        self.near_plane = near_plane;
        self
    }

    /// set the shadow bias
    ///
    /// default value: 0.0001
    pub fn bias(mut self, bias: f32) -> Self {
        self.bias = bias;
        self
    }
}
