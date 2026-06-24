use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use maple_engine::{
    asset::{Asset, AssetLibrary, AssetLoader},
    platform::SendSync,
    prelude::Resource,
};
use maple_renderer::{
    core::{
        AlphaMode as PipelineAlphaMode, CullMode, DepthCompare, DepthStencilOptions, DescriptorSet,
        DescriptorSetLayout, GraphicsShader, PipelineLayout, RenderContext, RenderDevice,
        RenderPipeline, texture::TextureFormat,
    },
    render_graph::node::DepthMode,
    shader_asset::ShaderSource,
};

#[derive(Clone)]
pub struct PassInfo {
    pub color_formats: Vec<TextureFormat>,
    pub sample_count: u32,
}

pub trait MaterialInstance: SendSync
where
    Self: 'static,
{
    fn vertex_shader() -> ShaderSource
    where
        Self: Sized;
    fn fragment_shader() -> ShaderSource
    where
        Self: Sized;
    fn alpha_mode(&self) -> AlphaMode;
    fn layout(&self, rcx: &RenderContext) -> DescriptorSetLayout;
    fn cull_mode(&self) -> CullMode {
        CullMode::Back
    }
    fn pipeline(
        &self,
        rcx: &RenderContext,
        pass_info: &PassInfo,
        pipeline_layout: PipelineLayout,
        shader: GraphicsShader,
    ) -> RenderPipeline {
        let (blend_mode, pipeline_alpha_mode) = match self.alpha_mode() {
            AlphaMode::Opaque | AlphaMode::Mask => (
                DepthMode::Texture(DepthStencilOptions {
                    format: TextureFormat::Depth32,
                    compare: DepthCompare::Less,
                    write_enabled: true,
                    depth_bias: None,
                }),
                PipelineAlphaMode::Opaque,
            ),
            AlphaMode::Blend => (
                DepthMode::Texture(DepthStencilOptions {
                    format: TextureFormat::Depth32,
                    compare: DepthCompare::Less,
                    write_enabled: false,
                    depth_bias: None,
                }),
                PipelineAlphaMode::Blend,
            ),
        };
        rcx.device()
            .create_pipeline(maple_renderer::core::PipelineCreateInfo {
                label: None,
                layout: pipeline_layout,
                shader,
                color_formats: &pass_info.color_formats,
                depth: blend_mode,
                cull_mode: self.cull_mode(),
                alpha_mode: pipeline_alpha_mode,
                sample_count: pass_info.sample_count,
                use_vertex_buffer: true,
            })
    }

    fn descriptor_set(
        &self,
        assets: &AssetLibrary,
        rcx: &RenderContext,
        layout: &DescriptorSetLayout,
    ) -> MaterialDescriptorState;
}

pub enum PipelineStage {
    Opaque,
    Transparent,
}

/// how to treat alpha channel for fragment colors
#[derive(Debug, Clone, PartialEq, Copy, Default)]
pub enum AlphaMode {
    /// mesh is opaque (cant see through it)
    #[default]
    Opaque,
    /// mesh is opaque to a point before being culled
    Mask,
    /// mesh opacity is same as alpha
    Blend,
}

#[derive(Default)]
pub struct MaterialPipelineCache {
    pub opaque: HashMap<TypeId, RenderPipeline>,
    pub transparent: HashMap<TypeId, RenderPipeline>,
}

impl Resource for MaterialPipelineCache {}

pub struct Material {
    instance: Arc<dyn MaterialInstance>,
    vertex_shader: ShaderSource,
    fragment_shader: ShaderSource,
}

impl Material {
    pub fn new<T: MaterialInstance + 'static>(instance: T) -> Self {
        Self {
            instance: Arc::new(instance),
            vertex_shader: T::vertex_shader(),
            fragment_shader: T::fragment_shader(),
        }
    }

    pub fn material_key(&self) -> TypeId {
        self.instance.type_id()
    }

    pub fn vertex_shader(&self) -> ShaderSource {
        self.vertex_shader
    }

    pub fn fragment_shader(&self) -> ShaderSource {
        self.fragment_shader
    }

    pub fn alpha_mode(&self) -> AlphaMode {
        self.instance.alpha_mode()
    }

    pub fn layout(&self, rcx: &RenderContext) -> DescriptorSetLayout {
        self.instance.layout(rcx)
    }

    pub fn cull_mode(&self) -> CullMode {
        self.instance.cull_mode()
    }

    pub fn pipeline(
        &self,
        rcx: &RenderContext,
        pass_info: &PassInfo,
        pipeline_layout: PipelineLayout,
        shader: GraphicsShader,
    ) -> RenderPipeline {
        self.instance
            .pipeline(rcx, pass_info, pipeline_layout, shader)
    }

    pub fn descriptor_set(
        &self,
        assets: &AssetLibrary,
        rcx: &RenderContext,
        layout: &DescriptorSetLayout,
    ) -> MaterialDescriptorState {
        self.instance.descriptor_set(assets, rcx, layout)
    }
}

/// lets the pass know the current state of the material if it requests it before textures are
/// loading
pub enum MaterialDescriptorState {
    Loading,
    Ready(DescriptorSet),
}

impl Into<Option<DescriptorSet>> for MaterialDescriptorState {
    fn into(self) -> Option<DescriptorSet> {
        match self {
            Self::Loading => None,
            Self::Ready(set) => Some(set),
        }
    }
}

impl Asset for Material {
    type Loader = MaterialLoader;
}

pub struct PipelineCache {
    opaque: HashMap<TypeId, RenderPipeline>,
}

pub struct MaterialLoader {
    pub device: RenderDevice,
}

impl AssetLoader for MaterialLoader {
    type Asset = Material;
}

impl MaterialLoader {
    pub fn new(device: RenderDevice) -> Self {
        Self { device }
    }
}
