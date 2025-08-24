use bitflags::bitflags;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindingResource, BindingType, Device, ShaderStages,
};

use crate::core::buffer::Buffer;

bitflags! {
    #[derive(Clone, Copy)]
    pub struct StageFlags: u32 {
        const VERTEX = 0b01;
        const FRAGMENT = 0b10;
    }
}

impl From<StageFlags> for ShaderStages {
    fn from(value: StageFlags) -> Self {
        let mut s = ShaderStages::empty();
        if value.contains(StageFlags::VERTEX) {
            s |= ShaderStages::VERTEX
        }
        if value.contains(StageFlags::FRAGMENT) {
            s |= ShaderStages::FRAGMENT
        }
        s
    }
}

pub struct DescriptorSetLayoutDescriptor<'a> {
    pub label: Option<&'a str>,
    pub visibility: StageFlags,
    pub layout: &'a [DescriptorBindingType],
}

#[derive(Clone, Debug)]
pub struct DescriptorSetLayout {
    pub(crate) backend: BindGroupLayout,
}

impl DescriptorSetLayout {
    pub fn create(device: &Device, info: DescriptorSetLayoutDescriptor) -> Self {
        let mut entries: Vec<wgpu::BindGroupLayoutEntry> = Vec::new();

        for (i, entry) in info.layout.iter().enumerate() {
            match entry {
                DescriptorBindingType::UniformBuffer => entries.push(wgpu::BindGroupLayoutEntry {
                    binding: i as u32,
                    visibility: info.visibility.into(),
                    ty: BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }),
            }
        }

        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &entries,
            label: info.label,
        });

        DescriptorSetLayout { backend: layout }
    }
}

pub struct DescriptorSetDescriptor<'a, T> {
    pub label: Option<&'a str>,
    pub layout: &'a DescriptorSetLayout,
    pub writes: &'a [DescriptorWrite<T>],
}

#[derive(Clone, Debug)]
pub struct DescriptorSet {
    pub(crate) backend: BindGroup,
}

impl DescriptorSet {
    pub fn builder() -> DescriptorSetBuilder {
        todo!()
    }

    pub fn new<T>(device: &Device, info: DescriptorSetDescriptor<T>) -> DescriptorSet {
        let mut entries = Vec::new();

        for entry in info.writes {
            match entry {
                DescriptorWrite::UniformBuffer { binding, buffer } => {
                    entries.push(BindGroupEntry {
                        binding: *binding,
                        resource: BindingResource::Buffer(buffer.buffer.as_entire_buffer_binding()),
                    })
                }
            }
        }

        let group = device.create_bind_group(&BindGroupDescriptor {
            layout: &info.layout.backend,
            entries: &entries,
            label: info.label,
        });

        DescriptorSet { backend: group }
    }
}

pub struct DescriptorSetBuilder {}

impl DescriptorSetBuilder {
    pub fn build(&self) -> DescriptorSet {
        todo!()
    }
}

pub enum DescriptorWrite<T> {
    UniformBuffer { binding: u32, buffer: Buffer<T> },
}

pub enum DescriptorBindingType {
    UniformBuffer,
}

pub struct DescriptorBindingDesc {
    /// binding location
    pub binding: u32,
    /// type of binding
    pub bindig_type: DescriptorBindingType,
    /// what stages of the shader are you binding to
    pub stages: StageFlags,
    /// array size
    pub count: u32,
}
