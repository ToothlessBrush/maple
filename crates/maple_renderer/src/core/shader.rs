use wgpu::{Device, ShaderModule, ShaderStages};

use crate::shader_asset::Shader;

// #[derive(Clone, PartialEq, Eq, Hash, Debug)]
// pub struct GraphicsShader {
//     pub(crate) vertex: ShaderModule,
//     pub(crate) fragment: ShaderModule,
// }

#[derive(Clone, Debug)]
pub struct GraphicsShader {
    pub vertex: Shader,
    pub fragment: Shader,
}

pub struct ComputeShader {
    pub(crate) inner: ShaderModule,
}

pub enum ShaderPair<'a> {
    Wgsl { vert: &'a str, frag: &'a str },
    Glsl { vert: &'a str, frag: &'a str },
    Spirv { vert: &'a [u8], frag: &'a [u8] },
}

pub enum ComputeShaderSource<'a> {
    Wgsl(&'a str),
    Glsl(&'a str),
    Sirv(&'a [u8]),
}

pub enum ShaderLang {
    Wgsl,
    Glsl,
    Spirv,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderStage {
    Vertex,
    Fragment,
}

impl Into<wgpu::ShaderStages> for ShaderStage {
    fn into(self) -> wgpu::ShaderStages {
        match self {
            ShaderStage::Vertex => return ShaderStages::VERTEX,
            ShaderStage::Fragment => return ShaderStages::FRAGMENT,
        }
    }
}

impl Into<wgpu::naga::ShaderStage> for ShaderStage {
    fn into(self) -> wgpu::naga::ShaderStage {
        match self {
            ShaderStage::Vertex => return wgpu::naga::ShaderStage::Vertex,
            ShaderStage::Fragment => return wgpu::naga::ShaderStage::Fragment,
        }
    }
}

impl ComputeShader {
    pub fn from_source(
        device: &Device,
        source: ComputeShaderSource<'_>,
        label: Option<&str>,
    ) -> Self {
        let shader_source = match source {
            ComputeShaderSource::Wgsl(code) => wgpu::ShaderSource::Wgsl(code.into()),
            ComputeShaderSource::Glsl(code) => wgpu::ShaderSource::Glsl {
                shader: code.into(),
                stage: wgpu::naga::ShaderStage::Compute,
                defines: &[],
            },
            ComputeShaderSource::Sirv(bytes) => {
                let u32_data: Vec<u32> = bytes
                    .chunks_exact(4)
                    .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .collect();
                wgpu::ShaderSource::SpirV(u32_data.into())
            }
        };

        let inner = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: label.or(Some("Compute Shader")),
            source: shader_source,
        });

        Self { inner }
    }
}
