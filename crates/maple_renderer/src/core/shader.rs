use std::fs;
use std::{borrow::Cow, path::Path};

use anyhow::{Context, Result, anyhow, bail};
use bytemuck::cast_vec;
use wgpu::{Device, ShaderModule, ShaderModuleDescriptor, ShaderSource, ShaderStages};

#[derive(Clone)]
pub struct GraphicsShader {
    pub(crate) vertex: ShaderModule,
    pub(crate) fragment: ShaderModule,
}

pub enum ShaderPair<'a> {
    Wgsl { vert: &'a str, frag: &'a str },
    Glsl { vert: &'a str, frag: &'a str },
    Spirv { vert: &'a [u32], frag: &'a [u32] },
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
            ShaderPair::Spirv { vert, frag } => (
                wgpu::ShaderSource::SpirV(vert.into()),
                wgpu::ShaderSource::SpirV(frag.into()),
            ),
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
                // Own the u32 words so the slices live long enough for the call
                let v_words = cast_vec::<u8, u32>(v_bytes);
                let f_words = cast_vec::<u8, u32>(f_bytes);
                Ok(Self::from_pair(
                    device,
                    ShaderPair::Spirv {
                        vert: &v_words,
                        frag: &f_words,
                    },
                ))
            }
        }
    }
}
