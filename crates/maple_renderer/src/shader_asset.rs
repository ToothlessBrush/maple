use maple_engine::asset::{Asset, AssetLoader, IntoAsset, LoadErr};

use crate::core::{RenderDevice, ShaderStage};

/// Program that runs on the GPU
#[derive(Debug, Clone)]
pub struct Shader {
    pub(crate) module: wgpu::ShaderModule,
    pub(crate) entry_point: Option<&'static str>,
}

impl Asset for Shader {
    type Loader = ShaderLoader;
}

impl Shader {
    pub(crate) fn create(
        device: &RenderDevice,
        entry_point: Option<&'static str>,
        descriptor: wgpu::ShaderModuleDescriptor,
    ) -> Self {
        let module = device.device.create_shader_module(descriptor);
        Self {
            module: module,
            entry_point,
        }
    }

    pub(crate) fn compile(device: &RenderDevice, shader: ShaderSource) -> Result<Self, LoadErr> {
        let source = match shader.source {
            EmbeddedSource::Wgsl(code) => wgpu::ShaderSource::Wgsl(code.into()),
            EmbeddedSource::Glsl { source, stage } => wgpu::ShaderSource::Glsl {
                shader: source.into(),
                stage: stage.into(),
                defines: &[],
            },
            EmbeddedSource::Spirv(bytes) => {
                if bytes.len() % 4 != 0 {
                    return Err(LoadErr::Import("SPIR-V length not divisible by 4".into()));
                }
                let words: Vec<u32> = bytes
                    .chunks_exact(4)
                    .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                    .collect();
                wgpu::ShaderSource::SpirV(words.into())
            }
        };

        Ok(Shader::create(
            device,
            shader.entry_point,
            wgpu::ShaderModuleDescriptor {
                label: shader.label,
                source,
            },
        ))
    }
}

/// Asset loader for [`Shader`]
pub struct ShaderLoader {
    pub(crate) device: RenderDevice,
}

impl AssetLoader for ShaderLoader {
    type Asset = Shader;
}

/// Embedded source code of a [`Shader`]
#[derive(Debug, Clone, Copy)]
pub enum EmbeddedSource {
    /// Wgsl source
    Wgsl(&'static str),
    /// glsl source
    Glsl {
        /// source code
        source: &'static str,
        /// shader stage
        stage: ShaderStage,
    },
    /// spirv source
    Spirv(&'static [u8]),
}

/// Source of the shader code
#[derive(Debug, Clone, Copy)]
pub struct ShaderSource {
    /// debug label for shader
    pub label: Option<&'static str>,
    /// entry point for shader
    pub entry_point: Option<&'static str>,
    /// source code of shader
    pub source: EmbeddedSource,
}

impl Into<ShaderSource> for &'static str {
    fn into(self) -> ShaderSource {
        ShaderSource {
            label: None,
            entry_point: None,
            source: EmbeddedSource::Wgsl(self),
        }
    }
}

impl Into<ShaderSource> for EmbeddedSource {
    fn into(self) -> ShaderSource {
        ShaderSource {
            label: None,
            entry_point: None,
            source: self,
        }
    }
}

impl IntoAsset<Shader> for ShaderSource {
    fn into_asset(
        self,
        loader: &<Shader as Asset>::Loader,
        _library: &maple_engine::prelude::AssetLibrary,
    ) -> Result<Shader, maple_engine::asset::LoadErr> {
        Shader::compile(&loader.device, self)
    }
}
