use super::{LazyBufferable, texture};
use crate::platform::SendSync;
use crate::shader_asset::{Shader, ShaderSource};
use crate::{
    core::{
        ComputeShader, ComputeShaderSource, DescriptorSetBuilder, ShaderPair,
        buffer::Buffer,
        descriptor_set::{DescriptorSet, DescriptorSetLayout, DescriptorSetLayoutDescriptor},
        pipeline::{
            ComputePipeline, ComputePipelineCreateInfo, PipelineCreateInfo, PipelineLayout,
            RenderPipeline,
        },
        texture::{
            Sampler, SamplerOptions, Texture, TextureCreateInfo, TextureCube, TextureCubeCreateInfo,
        },
    },
    types::Vertex,
};
use anyhow::Result;
use bytemuck::Pod;
use maple_engine::asset::LoadErr;
use std::sync::Arc;
use wgpu::{BufferUsages, Device, Limits, Queue};

/// Represents the rendering device (gpu) used for resource allocation
#[derive(Clone, Debug)]
pub struct RenderDevice {
    pub(crate) device: Arc<Device>,
    pub(crate) queue: Arc<Queue>,
}

impl RenderDevice {
    pub fn create_vertex_buffer(&self, vertices: &[Vertex]) -> Buffer<[Vertex]> {
        Buffer::from_slice(
            &self.device,
            vertices,
            BufferUsages::VERTEX,
            "Vertex Buffer",
        )
    }

    pub fn create_index_buffer(&self, indices: &[u32]) -> Buffer<[u32]> {
        Buffer::from_slice(&self.device, indices, BufferUsages::INDEX, "Index Buffer")
    }

    pub fn create_uniform_buffer<T: Pod + SendSync>(&self, uniform: &T) -> Buffer<T> {
        Buffer::from(
            &self.device,
            uniform,
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            "Uniform Buffer",
        )
    }

    pub fn create_storage_buffer<T: Pod + SendSync>(&self, data: &T) -> Buffer<T> {
        Buffer::from(
            &self.device,
            data,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            "Storage Buffer",
        )
    }

    pub fn create_empty_storage_buffer<T: Pod + SendSync>(&self) -> Buffer<T> {
        Buffer::empty(
            &self.device,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            "storage buffer",
        )
    }

    pub fn create_storage_buffer_slice<T: Pod + SendSync>(&self, data: &[T]) -> Buffer<[T]> {
        Buffer::from_slice(
            &self.device,
            data,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            "Storage Buffer",
        )
    }

    pub fn create_sized_storage_buffer<T: Pod + SendSync>(&self, len: usize) -> Buffer<[T]> {
        Buffer::from_size(
            &self.device,
            len,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            "storage buffer",
        )
    }

    pub fn get_buffer_from_lazy<T, B>(&self, lazy_buffer: &B) -> Buffer<T>
    where
        B: LazyBufferable<T>,
        T: ?Sized + SendSync,
    {
        lazy_buffer.get_buffer(&self.device, &self.queue)
    }

    pub fn create_texture(&self, info: TextureCreateInfo) -> Texture {
        Texture::create(&self.device, &info)
    }

    pub fn create_texture_cube(&self, info: TextureCubeCreateInfo) -> TextureCube {
        TextureCube::create(&self.device, &info)
    }

    pub fn create_texture_array(
        &self,
        info: texture::TextureArrayCreateInfo,
    ) -> texture::TextureArray {
        texture::TextureArray::create(&self.device, &info)
    }

    pub fn create_texture_cube_array(
        &self,
        info: texture::TextureCubeArrayCreateInfo,
    ) -> texture::TextureCubeArray {
        texture::TextureCubeArray::create(&self.device, &info)
    }

    pub fn create_sampler(&self, options: SamplerOptions) -> Sampler {
        Texture::create_sampler(&self.device, options)
    }

    pub fn load_texture_from_bytes(
        &self,
        bytes: &[u8],
        label: Option<&'static str>,
    ) -> Result<Texture, image::ImageError> {
        Texture::from_bytes(&self.device, &self.queue, bytes, label)
    }

    pub fn load_texture_from_file(
        &self,
        path: impl AsRef<std::path::Path>,
        label: Option<&'static str>,
    ) -> Result<Texture, image::ImageError> {
        Texture::from_file(&self.device, &self.queue, path, label)
    }

    pub fn create_descriptor_set_layout(
        &self,
        info: DescriptorSetLayoutDescriptor,
    ) -> DescriptorSetLayout {
        DescriptorSetLayout::create(&self.device, info)
    }

    pub fn build_descriptor_set(&self, builder: &DescriptorSetBuilder) -> DescriptorSet {
        builder.build(&self.device)
    }

    pub fn compile_shader(&self, shader: ShaderSource) -> Result<Shader, LoadErr> {
        Shader::compile(self, shader)
    }

    pub fn create_render_pipeline_layout(
        &self,
        descriptor_set_layouts: &[DescriptorSetLayout],
    ) -> PipelineLayout {
        PipelineLayout::create(&self.device, descriptor_set_layouts)
    }

    pub fn create_render_pipeline(
        &self,
        pipeline_create_info: PipelineCreateInfo,
    ) -> RenderPipeline {
        RenderPipeline::create(&self.device, pipeline_create_info)
    }

    // Convenience aliases for shorter method names
    pub fn create_pipeline_layout(&self, layouts: &[DescriptorSetLayout]) -> PipelineLayout {
        self.create_render_pipeline_layout(layouts)
    }

    pub fn create_pipeline(&self, create_info: PipelineCreateInfo) -> RenderPipeline {
        self.create_render_pipeline(create_info)
    }

    pub fn create_compute_shader(&self, source: ComputeShaderSource) -> ComputeShader {
        ComputeShader::from_source(&self.device, source, None)
    }

    pub fn create_compute_pipeline(&self, info: ComputePipelineCreateInfo) -> ComputePipeline {
        ComputePipeline::create(&self.device, info)
    }

    pub fn limits(&self) -> DeviceLimits {
        DeviceLimits {
            inner: self.device.limits(),
        }
    }
}

pub struct DeviceLimits {
    inner: Limits,
}

impl DeviceLimits {
    pub fn min_storage_buffer_alignment(&self) -> u32 {
        self.inner.min_storage_buffer_offset_alignment
    }
}
