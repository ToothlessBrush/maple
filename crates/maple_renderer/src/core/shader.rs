use std::path::Path;
use std::{fs, sync::Arc};

use anyhow::{Context, Result, bail};
use maple_engine::asset::{Asset, AssetLoader, LoadErr};
use wgpu::{Device, ShaderModule, ShaderModuleDescriptor, ShaderSource};

#[derive(Clone)]
pub struct GraphicsShader {
    pub(crate) vertex: ShaderModule,
    pub(crate) fragment: ShaderModule,
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

impl GraphicsShader {
    /// create a shader from a pair which contains the source for the 2 stages
    pub fn from_pair(device: &wgpu::Device, pair: ShaderPair<'_>) -> Self {
        let (vs_source, fs_source) = match pair {
            ShaderPair::Wgsl { vert, frag } => (
                wgpu::ShaderSource::Wgsl(vert.into()),
                wgpu::ShaderSource::Wgsl(frag.into()),
            ),
            ShaderPair::Glsl { vert, frag } => (
                wgpu::ShaderSource::Glsl {
                    shader: vert.into(),
                    stage: wgpu::naga::ShaderStage::Vertex,
                    defines: &[],
                },
                wgpu::ShaderSource::Glsl {
                    shader: frag.into(),
                    stage: wgpu::naga::ShaderStage::Fragment,
                    defines: &[],
                },
            ),
            ShaderPair::Spirv { vert, frag } => {
                let vert_u32: Vec<u32> = vert
                    .chunks_exact(4)
                    .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .collect();
                let frag_u32: Vec<u32> = frag
                    .chunks_exact(4)
                    .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .collect();
                (
                    wgpu::ShaderSource::SpirV(vert_u32.into()),
                    wgpu::ShaderSource::SpirV(frag_u32.into()),
                )
            }
        };

        let vertex = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("vertex module"),
            source: vs_source,
        });
        let fragment = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fragment module"),
            source: fs_source,
        });

        Self { vertex, fragment }
    }

    /// create a shader in a specified language from a path
    pub fn from_path(
        device: &Device,
        lang: ShaderLang,
        vert_path: &Path,
        frag_path: &Path,
    ) -> Result<Self> {
        match lang {
            ShaderLang::Wgsl => {
                let v = fs::read_to_string(vert_path)
                    .with_context(|| format!("reading WGSL vertex: {}", vert_path.display()))?;
                let f = fs::read_to_string(frag_path)
                    .with_context(|| format!("reading WGSL fragment: {}", frag_path.display()))?;
                Ok(Self::from_pair(
                    device,
                    ShaderPair::Wgsl { vert: &v, frag: &f },
                ))
            }
            ShaderLang::Glsl => {
                let v = fs::read_to_string(vert_path)
                    .with_context(|| format!("reading GLSL vertex: {}", vert_path.display()))?;
                let f = fs::read_to_string(frag_path)
                    .with_context(|| format!("reading GLSL fragment: {}", frag_path.display()))?;
                Ok(Self::from_pair(
                    device,
                    ShaderPair::Glsl { vert: &v, frag: &f },
                ))
            }
            ShaderLang::Spirv => {
                let v_bytes = fs::read(vert_path)
                    .with_context(|| format!("reading SPIR-V vertex: {}", vert_path.display()))?;
                let f_bytes = fs::read(frag_path)
                    .with_context(|| format!("reading SPIR-V fragment: {}", frag_path.display()))?;
                if v_bytes.len() % 4 != 0 || f_bytes.len() % 4 != 0 {
                    bail!("SPIR-V files must have lengths divisible by 4");
                }
                Ok(Self::from_pair(
                    device,
                    ShaderPair::Spirv {
                        vert: &v_bytes,
                        frag: &f_bytes,
                    },
                ))
            }
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
