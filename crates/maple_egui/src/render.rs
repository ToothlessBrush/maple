use std::collections::HashMap;

use bytemuck::{Pod, Zeroable};
use egui::{ImageData, TextureId, epaint::ImageDelta};
use maple_engine::prelude::Input;
use maple_renderer::{
    core::{
        Buffer, DescriptorBindingType, DescriptorSet, DescriptorSetLayout,
        DescriptorSetLayoutDescriptor, GraphicsShader, PipelineCreateInfo, PipelineLayout,
        RenderContext, RenderPipeline, StageFlags,
        texture::{Sampler, Texture, TextureMode, TextureUsage},
    },
    render_graph::{
        graph::{RenderGraphContext, Stage},
        node::{RenderNode, RenderTarget},
    },
    shader_asset::ShaderSource,
    types::vertex::{VertexLayout, vertex_attr_array},
};

use crate::plugin::EguiResource;

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, Pod, Zeroable)]
struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2],
    color: u32,
}

impl VertexLayout for Vertex {
    const ATTRS: &'static [maple_renderer::types::vertex::VertexAttribute] = &vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2,
        2 => Uint32,
    ];
}

impl From<egui::epaint::Vertex> for Vertex {
    fn from(v: egui::epaint::Vertex) -> Self {
        Self {
            pos: [v.pos.x, v.pos.y],
            uv: [v.uv.x, v.uv.y],
            color: u32::from_le_bytes(v.color.to_array()),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Locals {
    pub screen_size: [f32; 2],
    pub dithering: u32,
    pub predictable_texture_filtering: u32,
}

pub struct EguiTexture {
    texture: Texture,
    descriptor: DescriptorSet,
}

pub struct EguiRender {
    pipeline: RenderPipeline,
    texture_layout: DescriptorSetLayout,
    sampler: Sampler,
    textures: HashMap<TextureId, EguiTexture>,

    local_layout: DescriptorSetLayout,
    local_buffer: Buffer<Locals>,
    local_descriptor: DescriptorSet,

    vertex_buffer: Buffer<[Vertex]>,
    vertex_capacity: usize,
    index_buffer: Buffer<[u32]>,
    index_capacity: usize,
}

impl RenderNode for EguiRender {
    fn stage(&self) -> Stage {
        Stage::Ui
    }
    fn setup(rcx: &RenderContext, graph_ctx: &mut RenderGraphContext) -> Self
    where
        Self: Sized,
    {
        let fragment = rcx
            .device()
            .compile_shader(ShaderSource {
                source: maple_renderer::shader_asset::EmbeddedSource::Wgsl(include_str!(
                    "egui.wgsl"
                )),
                label: Some("egui shader"),
                entry_point: Some("fs_main_linear_framebuffer"),
            })
            .expect("failed to compile egui fragment shader");
        let vertex = rcx
            .device()
            .compile_shader(include_str!("egui.wgsl").into())
            .expect("failed to compile egui vertex shader");
        let shader = GraphicsShader {
            vertex: vertex,
            fragment: fragment,
        };

        let texture_layout =
            rcx.device()
                .create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
                    label: Some("egui texture"),
                    visibility: StageFlags::FRAGMENT,
                    layout: &[
                        DescriptorBindingType::TextureView { filterable: true },
                        DescriptorBindingType::Sampler { filtering: true },
                    ],
                });

        let local_layout =
            rcx.device()
                .create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
                    label: Some("egui screen size"),
                    visibility: StageFlags::VERTEX | StageFlags::FRAGMENT,
                    layout: &[DescriptorBindingType::UniformBuffer],
                });

        let local_buffer = rcx.device().create_uniform_buffer(&Locals {
            screen_size: [0.0, 0.0],
            dithering: 1,
            predictable_texture_filtering: 0,
        });

        let local_descriptor = rcx.device().build_descriptor_set(
            DescriptorSet::builder(&local_layout)
                .label("local descriptor")
                .uniform(0, &local_buffer),
        );

        let sampler = rcx
            .device()
            .create_sampler(maple_renderer::core::texture::SamplerOptions {
                mode_u: TextureMode::ClampToEdge,
                mode_v: TextureMode::ClampToEdge,
                mode_w: TextureMode::ClampToEdge,
                mag_filter: maple_renderer::core::texture::FilterMode::Linear,
                min_filter: maple_renderer::core::texture::FilterMode::Linear,
                compare: None,
            });

        let surface_format = rcx.surface_format();
        let pipeline = rcx.device().create_render_pipeline(PipelineCreateInfo {
            shader,
            label: Some("egui"),
            alpha_mode: maple_renderer::core::AlphaMode::Blend,
            color_formats: &[surface_format],
            cull_mode: maple_renderer::core::CullMode::None,
            depth: maple_renderer::render_graph::node::DepthMode::None,
            layout: rcx
                .device()
                .create_pipeline_layout(&[local_layout.clone(), texture_layout.clone()]),
            sample_count: 1,
            vertex_buffer_layout: Some(Vertex::buffer_layout()),
        });

        let initial_cap = 4096;

        Self {
            pipeline,
            texture_layout,
            sampler,
            textures: HashMap::new(),
            local_layout,
            local_buffer,
            local_descriptor,
            vertex_buffer: rcx
                .device()
                .create_sized_vertex_buffer(initial_cap * size_of::<Vertex>()),
            vertex_capacity: initial_cap,
            index_buffer: rcx
                .device()
                .create_sized_index_buffer(initial_cap * size_of::<u32>()),
            index_capacity: initial_cap,
        }
    }

    fn draw(
        &mut self,
        rcx: &RenderContext,
        frame: &mut maple_renderer::core::Frame,
        graph_ctx: &mut RenderGraphContext,
        game_ctx: &maple_engine::GameContext,
    ) {
        let mut egui_res = game_ctx.get_resource_mut::<EguiResource>();
        let Some(full_output) = egui_res.full_output.take() else {
            return;
        };

        for (id, image_delta) in &full_output.textures_delta.set {
            self.update_texture(rcx, *id, image_delta);
        }

        let clipped_primitives = egui_res
            .context
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        let (vertices, indices, mesh_ranges) = Self::flatten_primitives(&clipped_primitives);
        if vertices.is_empty() {
            for id in &full_output.textures_delta.free {
                self.textures.remove(id);
            }
        }

        self.ensure_capacity(rcx, vertices.len(), indices.len());
        rcx.queue()
            .write_buffer_slice(&self.vertex_buffer, &vertices);
        rcx.queue().write_buffer_slice(&self.index_buffer, &indices);

        let input = game_ctx.get_resource::<Input>();
        let screen_size = input.screen_size_points();
        rcx.queue().write_buffer(
            &self.local_buffer,
            &Locals {
                screen_size: [screen_size.x, screen_size.y],
                dithering: 1,
                predictable_texture_filtering: 0,
            },
        );

        let scale = input.scale_factor();

        frame
            .render(
                maple_renderer::core::context::RenderOptions {
                    label: Some("Egui Pass"),
                    color_targets: &[RenderTarget::Surface],
                    depth_target: None,
                    clear_color: None,
                    clear_depth: None,
                },
                move |mut fb| {
                    fb.use_pipeline(&self.pipeline)
                        .bind_descriptor_set(0, &self.local_descriptor);

                    for (index_range, clip_rect, texture_id) in &mesh_ranges {
                        let Some(tex) = self.textures.get(texture_id) else {
                            continue;
                        };
                        fb.bind_descriptor_set(1, &tex.descriptor);

                        fb.set_scissor_rect(
                            (clip_rect.min.x * scale) as u32,
                            (clip_rect.min.y * scale) as u32,
                            (clip_rect.width() * scale) as u32,
                            (clip_rect.height() * scale) as u32,
                        );

                        fb.bind_vertex_buffer(&self.vertex_buffer)
                            .bind_index_buffer(&self.index_buffer)
                            .draw_indexed_range(index_range.clone());
                    }
                },
            )
            .expect("failed to render egui");
    }
}

impl EguiRender {
    fn update_texture(&mut self, rcx: &RenderContext, id: TextureId, delta: &ImageDelta) {
        let pixels: Vec<u8> = match &delta.image {
            ImageData::Color(image) => image.pixels.iter().flat_map(|c| c.to_array()).collect(),
        };

        match delta.pos {
            Some(pos) => {
                let existing = self
                    .textures
                    .get(&id)
                    .expect("partial update on missing texture");
                rcx.queue().write_texture_reigon(
                    &existing.texture,
                    &pixels,
                    pos[0] as u32,
                    pos[1] as u32,
                    delta.image.width() as u32,
                    delta.image.height() as u32,
                );
            }
            None => {
                let texture =
                    rcx.device()
                        .create_texture(maple_renderer::core::texture::TextureCreateInfo {
                            label: Some("egui texture"),
                            width: delta.image.width() as u32,
                            height: delta.image.height() as u32,
                            format: maple_renderer::core::texture::TextureFormat::RGBA8,
                            usage: TextureUsage::TEXTURE_BINDING | TextureUsage::COPY_DST,
                            sample_count: 1,
                            mip_level: 1,
                        });
                rcx.queue().write_texture(&texture, &pixels);
                let descriptor = rcx.device().build_descriptor_set(
                    DescriptorSet::builder(&self.texture_layout)
                        .texture_view(0, &texture.create_view())
                        .sampler(1, &self.sampler),
                );
                self.textures.insert(
                    id,
                    EguiTexture {
                        texture,
                        descriptor,
                    },
                );
            }
        }
    }

    fn flatten_primitives(
        clipped_primitives: &[egui::ClippedPrimitive],
    ) -> (
        Vec<Vertex>,
        Vec<u32>,
        Vec<(std::ops::Range<u32>, egui::Rect, egui::TextureId)>,
    ) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut ranges = Vec::new();

        for prim in clipped_primitives {
            let egui::epaint::Primitive::Mesh(mesh) = &prim.primitive else {
                continue;
            };
            if mesh.vertices.is_empty() || mesh.indices.is_empty() {
                continue;
            }

            let vertex_base = vertices.len() as u32;
            let index_start = indices.len() as u32;

            vertices.extend(mesh.vertices.iter().map(|&v| Vertex::from(v)));
            indices.extend(mesh.indices.iter().map(|i| i + vertex_base));

            ranges.push((
                index_start..indices.len() as u32,
                prim.clip_rect,
                mesh.texture_id,
            ));
        }

        (vertices, indices, ranges)
    }
    fn ensure_capacity(&mut self, rcx: &RenderContext, vertex_count: usize, index_count: usize) {
        if vertex_count > self.vertex_capacity {
            self.vertex_capacity = vertex_count.next_power_of_two();
            self.vertex_buffer = rcx
                .device()
                .create_sized_vertex_buffer(self.vertex_capacity * size_of::<Vertex>());
        }
        if index_count > self.index_capacity {
            self.index_capacity = index_count.next_power_of_two();
            self.index_buffer = rcx
                .device()
                .create_sized_index_buffer(self.index_capacity * size_of::<u32>());
        }
    }
}
