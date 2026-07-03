use anyhow::Result;
use wgpu::{CommandEncoder, ComputePass, Operations, RenderPass, RenderPassDepthStencilAttachment};

use crate::{
    core::{
        ComputePipeline, RenderContext, RenderPipeline, buffer::Buffer, context::RenderOptions,
        descriptor_set::DescriptorSet,
    },
    render_graph::node::RenderTarget,
    types::Vertex,
};

pub struct Frame<'a> {
    pub(crate) encoder: CommandEncoder,
    pub(crate) renderer: &'a RenderContext,
}

impl Frame<'_> {
    pub fn render<F>(&mut self, options: RenderOptions, execute: F) -> Result<()>
    where
        F: FnOnce(FrameBuilder),
    {
        // Prepare the render target only as needed
        struct PreparedTarget {
            view: wgpu::TextureView,
            resolve_view: Option<wgpu::TextureView>,
        }

        let mut prepared = Vec::new();

        for target in options.color_targets {
            match target {
                RenderTarget::Surface => {
                    let surface_tex = self.renderer.get_surface_texture().unwrap();
                    let view = surface_tex
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    prepared.push(PreparedTarget {
                        view,
                        resolve_view: None,
                    });
                }
                RenderTarget::Texture(t) => {
                    prepared.push(PreparedTarget {
                        view: t.inner.clone(),
                        resolve_view: None,
                    });
                }
                RenderTarget::MultiSampled { texture, resolve } => prepared.push(PreparedTarget {
                    view: texture.inner.clone(),
                    resolve_view: Some(resolve.inner.clone()),
                }),
            }
        }

        let depth_view = options.depth_target;

        let depth_stencil_attachment =
            depth_view
                .as_ref()
                .map(|view| RenderPassDepthStencilAttachment {
                    view: &view.inner,
                    depth_ops: Some(Operations {
                        load: options
                            .clear_depth
                            .map(wgpu::LoadOp::Clear)
                            .unwrap_or(wgpu::LoadOp::Load),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                });

        let color_attachments: Vec<Option<wgpu::RenderPassColorAttachment>> = prepared
            .iter()
            .map(|prepared_target| {
                Some(wgpu::RenderPassColorAttachment {
                    view: &prepared_target.view,
                    resolve_target: prepared_target.resolve_view.as_ref(),
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: match options.clear_color {
                            Some([r, g, b, a]) => wgpu::LoadOp::Clear(wgpu::Color {
                                r: r as f64,
                                g: g as f64,
                                b: b as f64,
                                a: a as f64,
                            }),
                            None => wgpu::LoadOp::Load,
                        },
                        store: wgpu::StoreOp::Store,
                    },
                })
            })
            .collect();

        let render_pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: options.label,
            color_attachments: &color_attachments,
            depth_stencil_attachment,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        let frame_builder = FrameBuilder::new(render_pass);
        // where we build the user command buffer pass in bound
        // automatically by frame builder
        execute(frame_builder);

        // done rendering this pass

        Ok(())
    }

    pub fn compute<F>(&mut self, label: Option<&str>, execute: F)
    where
        F: FnOnce(ComputeBuilder),
    {
        let compute_pass = self
            .encoder
            .begin_compute_pass(&wgpu::ComputePassDescriptor {
                label,
                timestamp_writes: None,
            });

        let compute_builder = ComputeBuilder::new(compute_pass);
        execute(compute_builder);
    }
}

/// builder for a frame use this to bind buffers, descriptor sets, or anything else frame related
///
/// since the frame contains a refrence to the command encoder we need its lifetime
pub struct FrameBuilder<'encoder> {
    pub(crate) backend: RenderPass<'encoder>,
    index_count: u32,
    vertex_count: u32,
}

impl<'encoder> FrameBuilder<'encoder> {
    pub(crate) fn new(backend: RenderPass<'encoder>) -> Self {
        FrameBuilder {
            backend,
            index_count: 0,
            vertex_count: 0,
        }
    }

    pub fn use_pipeline(&mut self, pipeline: &RenderPipeline) -> &mut Self {
        self.backend.set_pipeline(&pipeline.backend);

        self
    }

    /// vertex buffer for the next draw call
    pub fn bind_vertex_buffer(&mut self, vertex_buffer: &Buffer<[Vertex]>) -> &mut Self {
        self.backend
            .set_vertex_buffer(0, vertex_buffer.buffer.slice(..));

        self.vertex_count = vertex_buffer.len() as u32;

        self
    }

    /// index buffer for the next draw_indexed call
    pub fn bind_index_buffer(&mut self, index_buffer: &Buffer<[u32]>) -> &mut Self {
        self.backend
            .set_index_buffer(index_buffer.buffer.slice(..), wgpu::IndexFormat::Uint32);

        self.index_count = index_buffer.len() as u32;

        self
    }

    // set a descriptor set must be in the pipeline layout
    pub fn bind_descriptor_set(&mut self, set: u32, descriptor_set: &DescriptorSet) -> &mut Self {
        self.backend
            .set_bind_group(set, &descriptor_set.backend, &[]);

        self
    }

    pub fn bind_descriptor_set_with_offset(
        &mut self,
        set: u32,
        descriptor_set: &DescriptorSet,
        offsets: &[u32],
    ) -> &mut Self {
        self.backend
            .set_bind_group(set, &descriptor_set.backend, offsets);

        self
    }

    pub fn debug_marker(&mut self, label: &str) -> &mut Self {
        self.backend.insert_debug_marker(label);

        self
    }

    /// draw the last bound indicies
    pub fn draw_indexed(&mut self) -> &mut Self {
        self.backend.draw_indexed(0..self.index_count, 0, 0..1);

        self
    }

    /// draw the last bound vertices
    pub fn draw_vertices(&mut self) -> &mut Self {
        self.backend.draw(0..self.vertex_count, 0..1);

        self
    }

    /// draw vertices with explicit vertex range (for vertex-less rendering like fullscreen triangles)
    pub fn draw(&mut self, vertices: std::ops::Range<u32>) -> &mut Self {
        self.backend.draw(vertices, 0..1);

        self
    }
}

pub struct ComputeBuilder<'encoder> {
    pub(crate) backend: ComputePass<'encoder>,
}

impl<'encoder> ComputeBuilder<'encoder> {
    pub(crate) fn new(backend: ComputePass<'encoder>) -> Self {
        Self { backend }
    }

    pub fn use_pipeline(&mut self, pipeline: &ComputePipeline) -> &mut Self {
        self.backend.set_pipeline(&pipeline.backend);
        self
    }

    pub fn bind_descriptor_set(&mut self, set: u32, descriptor_set: &DescriptorSet) -> &mut Self {
        self.backend
            .set_bind_group(set, &descriptor_set.backend, &[]);
        self
    }

    pub fn debug_marker(&mut self, label: &str) -> &mut Self {
        self.backend.insert_debug_marker(label);
        self
    }

    pub fn dispatch(&mut self, x: u32, y: u32, z: u32) -> &mut Self {
        self.backend.dispatch_workgroups(x, y, z);
        self
    }
}
