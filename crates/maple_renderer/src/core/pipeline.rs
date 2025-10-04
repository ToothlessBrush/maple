use wgpu::{
    BindGroupLayout, BlendState, ColorTargetState, ColorWrites, Device, Face, FragmentState,
    FrontFace, MultisampleState, PipelineCompilationOptions, PipelineLayoutDescriptor, PolygonMode,
    PrimitiveState, PrimitiveTopology, RenderPipelineDescriptor, VertexState,
};

use crate::{
    core::{descriptor_set::DescriptorSetLayout, shader::GraphicsShader},
    types::Vertex,
};

use super::texture::Texture;

pub struct PipelineLayout {
    pub(crate) backend: wgpu::PipelineLayout,
}

impl PipelineLayout {
    pub fn create(device: &Device, descriptor_set_layout: &[DescriptorSetLayout]) -> Self {
        let binding_layouts: Vec<&BindGroupLayout> =
            descriptor_set_layout.iter().map(|d| &d.backend).collect();

        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &binding_layouts,
            push_constant_ranges: &[],
        });

        PipelineLayout { backend: layout }
    }
}

#[derive(Clone)]
pub struct RenderPipeline {
    pub(crate) backend: wgpu::RenderPipeline,
}

#[derive(Clone, Copy, Debug)]
pub enum DepthCompare {
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
    Always,
    Never,
}

impl From<DepthCompare> for wgpu::CompareFunction {
    fn from(value: DepthCompare) -> Self {
        match value {
            DepthCompare::Less => Self::Less,
            DepthCompare::LessEqual => Self::LessEqual,
            DepthCompare::Greater => Self::Greater,
            DepthCompare::GreaterEqual => Self::GreaterEqual,
            DepthCompare::Equal => Self::Equal,
            DepthCompare::NotEqual => Self::NotEqual,
            DepthCompare::Always => Self::Always,
            DepthCompare::Never => Self::Never,
        }
    }
}

pub struct DepthStencilOptions {
    pub texture: Texture,
    pub compare: DepthCompare,
    pub write_enabled: bool,
}
impl DepthStencilOptions {
    pub fn new(texture: Texture) -> Self {
        Self {
            texture,
            compare: DepthCompare::Less,
            write_enabled: true,
        }
    }

    pub fn to_wgpu_state(&self) -> wgpu::DepthStencilState {
        wgpu::DepthStencilState {
            format: self.texture.format().into(),
            depth_write_enabled: self.write_enabled,
            depth_compare: self.compare.into(),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }
    }
}

pub struct PipelineCreateInfo<'a> {
    pub label: Option<&'static str>,
    pub layout: PipelineLayout,
    pub shader: GraphicsShader,
    pub color_format: crate::core::texture::TextureFormat,
    pub depth: Option<&'a DepthStencilOptions>,
}

impl RenderPipeline {
    pub fn create(device: &Device, pipeline_create_info: PipelineCreateInfo) -> Self {
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: pipeline_create_info.label,
            layout: Some(&pipeline_create_info.layout.backend),
            vertex: VertexState {
                module: &pipeline_create_info.shader.vertex,
                entry_point: Some("main"),
                buffers: &[Vertex::buffer_layout()],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &pipeline_create_info.shader.fragment,
                entry_point: Some("main"),
                targets: &[Some(ColorTargetState {
                    format: pipeline_create_info.color_format.into(),
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: pipeline_create_info
                .depth
                .map(|depth| depth.to_wgpu_state()),
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        RenderPipeline { backend: pipeline }
    }
}
