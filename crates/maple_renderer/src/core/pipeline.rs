use wgpu::{
    BindGroupLayout, BlendState, ColorTargetState, ColorWrites, Device, Face, FragmentState,
    FrontFace, MultisampleState, PipelineCompilationOptions, PipelineLayoutDescriptor, PolygonMode,
    PrimitiveState, PrimitiveTopology, RenderPipelineDescriptor, VertexState,
};

#[derive(Clone, Copy, Debug)]
pub enum CullMode {
    None,
    Front,
    Back,
}

impl From<CullMode> for Option<Face> {
    fn from(value: CullMode) -> Self {
        match value {
            CullMode::None => None,
            CullMode::Front => Some(Face::Front),
            CullMode::Back => Some(Face::Back),
        }
    }
}

use crate::{
    core::{descriptor_set::DescriptorSetLayout, shader::GraphicsShader},
    render_graph::node::DepthMode,
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

#[derive(Clone, Copy, Debug)]
pub enum AlphaMode {
    Opaque,
    Blend,
}

impl From<AlphaMode> for wgpu::BlendState {
    fn from(value: AlphaMode) -> Self {
        match value {
            AlphaMode::Opaque => Self::REPLACE,
            AlphaMode::Blend => Self::ALPHA_BLENDING,
        }
    }
}

pub struct DepthStencilOptions {
    pub texture: Texture,
    pub compare: DepthCompare,
    pub write_enabled: bool,
    pub depth_bias: Option<(f32, f32)>, // (constant, slope_scale)
}
impl DepthStencilOptions {
    pub fn new(texture: Texture) -> Self {
        Self {
            texture,
            compare: DepthCompare::Less,
            write_enabled: true,
            depth_bias: None,
        }
    }

    pub fn to_wgpu_state(&self) -> wgpu::DepthStencilState {
        let bias = if let Some((constant, slope_scale)) = self.depth_bias {
            wgpu::DepthBiasState {
                constant: constant as i32,
                slope_scale,
                clamp: 0.0,
            }
        } else {
            wgpu::DepthBiasState::default()
        };

        wgpu::DepthStencilState {
            format: self.texture.format().into(),
            depth_write_enabled: self.write_enabled,
            depth_compare: self.compare.into(),
            stencil: wgpu::StencilState::default(),
            bias,
        }
    }
}

pub struct PipelineCreateInfo<'a> {
    pub label: Option<&'static str>,
    pub layout: PipelineLayout,
    pub shader: GraphicsShader,
    pub color_format: Option<crate::core::texture::TextureFormat>,
    pub depth: &'a DepthMode,
    pub cull_mode: CullMode,
    pub alpha_mode: AlphaMode,
    pub sample_count: u32,
    pub use_vertex_buffer: bool,
}

impl RenderPipeline {
    pub fn create(device: &Device, pipeline_create_info: PipelineCreateInfo) -> Self {
        // Create color targets if color_format is provided, otherwise use empty slice for depth-only
        let color_target;
        let color_targets: &[Option<ColorTargetState>] = match pipeline_create_info.color_format {
            Some(format) => {
                color_target = Some(ColorTargetState {
                    format: format.into(),
                    blend: Some(pipeline_create_info.alpha_mode.into()),
                    write_mask: ColorWrites::ALL,
                });
                std::slice::from_ref(&color_target)
            }
            None => &[],
        };

        // Create vertex buffer layout if needed
        let vertex_buffer_layout;
        let vertex_buffers: &[_] = if pipeline_create_info.use_vertex_buffer {
            vertex_buffer_layout = Vertex::buffer_layout();
            std::slice::from_ref(&vertex_buffer_layout)
        } else {
            &[]
        };

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: pipeline_create_info.label,
            layout: Some(&pipeline_create_info.layout.backend),
            vertex: VertexState {
                module: &pipeline_create_info.shader.vertex,
                entry_point: Some("main"),
                buffers: vertex_buffers,
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &pipeline_create_info.shader.fragment,
                entry_point: Some("main"),
                targets: color_targets,
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: pipeline_create_info.cull_mode.into(),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: match pipeline_create_info.depth {
                DepthMode::None => None,
                DepthMode::Auto(options) => Some(options.to_wgpu_state()),
                DepthMode::Manual(options) => Some(options.to_wgpu_state()),
            },
            multisample: MultisampleState {
                count: pipeline_create_info.sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        RenderPipeline { backend: pipeline }
    }
}
