use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, OnceLock},
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
        RenderPipeline, descriptor_set, texture::TextureFormat,
    },
    render_graph::node::DepthMode,
    shader_asset::ShaderSource,
};

#[derive(Clone)]
pub struct PassInfo {
    pub color_formats: Vec<TextureFormat>,
    pub sample_count: u32,
}

pub trait GpuMateiral: SendSync {
    fn descriptor_set(&self) -> DescriptorSet;
}

pub trait MaterialInstance: SendSync + AsAny
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
    fn casts_shadows(&self) -> bool {
        true
    }
    fn label(&self) -> &'static str {
        "Material"
    }

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
                label: Some(self.label()),
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

    fn prepare(
        &self,
        rcx: &RenderContext,
        assets: &AssetLibrary,
        layout: &DescriptorSetLayout,
    ) -> Option<Arc<dyn GpuMateiral>>;
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

impl<T: MaterialInstance + 'static> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
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

// type erased Material Asset
pub struct Material {
    // TODO: mutable materials - I think a good way to do that would be to seperate the Instance
    // from the Gpu Resources which would mean there wouldnt need to be 2 types for the same
    // material one with data and the other with data + gpu. a problem is how we would
    // store buffer data in a way that allows the most freedom for material implementations
    instance: Arc<dyn MaterialInstance>,
    gpu_material: OnceLock<Arc<dyn GpuMateiral>>,
    vertex_shader: ShaderSource,
    fragment_shader: ShaderSource,
}

impl Material {
    pub fn new<T: MaterialInstance + 'static>(instance: T) -> Self {
        Self {
            instance: Arc::new(instance),
            gpu_material: OnceLock::new(),
            vertex_shader: T::vertex_shader(),
            fragment_shader: T::fragment_shader(),
        }
    }

    pub fn casts_shadows(&self) -> bool {
        self.instance.casts_shadows()
    }

    pub fn get_instance<T: MaterialInstance + 'static>(&self) -> Option<&T> {
        self.instance.as_any().downcast_ref()
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
        rcx: &RenderContext,
        assets: &AssetLibrary,
    ) -> Option<DescriptorSet> {
        match self.gpu_material.get().map(|mat| mat.descriptor_set()) {
            Some(descriptor_set) => Some(descriptor_set),
            None => {
                let layout = self.instance.layout(rcx);
                let Some(gpu_material) = self.instance.prepare(rcx, assets, &layout) else {
                    return None;
                };
                Some(
                    self.gpu_material
                        .get_or_init(|| gpu_material)
                        .descriptor_set(),
                )
            }
        }
    }
}

impl Asset for Material {
    type Loader = MaterialLoader;
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
