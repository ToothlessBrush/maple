use bitflags::bitflags;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindingResource, BindingType, Device, SamplerBindingType, ShaderStages, TextureSampleType,
};

use crate::core::{
    buffer::Buffer,
    texture::{Sampler, TextureView},
};

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
                DescriptorBindingType::TextureView => entries.push(wgpu::BindGroupLayoutEntry {
                    binding: i as u32,
                    visibility: info.visibility.into(),
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                }),
                DescriptorBindingType::Sampler => entries.push(wgpu::BindGroupLayoutEntry {
                    binding: i as u32,
                    visibility: info.visibility.into(),
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                }),
                DescriptorBindingType::Storage { read_only } => {
                    entries.push(wgpu::BindGroupLayoutEntry {
                        binding: i as u32,
                        visibility: info.visibility.into(),
                        ty: BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage {
                                read_only: *read_only,
                            },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    })
                }
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
    pub fn builder<'a>(layout: &'a DescriptorSetLayout) -> DescriptorSetBuilder<'a> {
        DescriptorSetBuilder {
            label: None,
            layout,
            entries: Vec::new(),
        }
    }

    // pub fn new<T>(device: &Device, info: DescriptorSetDescriptor<T>) -> DescriptorSet {
    //     let mut entries = Vec::new();

    //     for entry in info.writes {
    //         match entry {
    //             DescriptorWrite::UniformBuffer { binding, buffer } => {
    //                 entries.push(BindGroupEntry {
    //                     binding: *binding,
    //                     resource: BindingResource::Buffer(buffer.buffer.as_entire_buffer_binding()),
    //                 })
    //             }
    //         }
    //     }

    //     let group = device.create_bind_group(&BindGroupDescriptor {
    //         layout: &info.layout.backend,
    //         entries: &entries,
    //         label: info.label,
    //     });

    //     DescriptorSet { backend: group }
    // }
}

pub struct DescriptorSetBuilder<'a> {
    pub(crate) label: Option<&'a str>,
    pub(crate) layout: &'a DescriptorSetLayout,
    pub(crate) entries: Vec<BindGroupEntry<'a>>,
}

impl<'a> DescriptorSetBuilder<'a> {
    pub fn label(&mut self, label: &'a str) -> &mut Self {
        self.label = Some(label);

        self
    }

    pub fn uniform<T>(&mut self, binding: u32, buffer: &'a Buffer<T>) -> &mut Self {
        self.entries.push(BindGroupEntry {
            binding,
            resource: BindingResource::Buffer(buffer.buffer.as_entire_buffer_binding()),
        });

        self
    }

    pub fn texture_view(&mut self, binding: u32, view: &'a TextureView) -> &mut Self {
        self.entries.push(BindGroupEntry {
            binding,
            resource: BindingResource::TextureView(&view.inner),
        });

        self
    }

    pub fn sampler(&mut self, binding: u32, sampler: &'a Sampler) -> &mut Self {
        self.entries.push(BindGroupEntry {
            binding,
            resource: BindingResource::Sampler(&sampler.inner),
        });

        self
    }

    pub fn storage<T: ?Sized>(&mut self, binding: u32, storage_buffer: &'a Buffer<T>) -> &mut Self {
        self.entries.push(BindGroupEntry {
            binding,
            resource: BindingResource::Buffer(storage_buffer.buffer.as_entire_buffer_binding()),
        });

        self
    }

    pub fn write<T>(&mut self, binding: u32, write: &'a DescriptorWrite<T>) -> &mut Self {
        match write {
            DescriptorWrite::UniformBuffer(buffer) => self.entries.push(BindGroupEntry {
                binding,
                resource: BindingResource::Buffer(buffer.buffer.as_entire_buffer_binding()),
            }),
            DescriptorWrite::TextureView(view) => self.entries.push(BindGroupEntry {
                binding,
                resource: BindingResource::TextureView(&view.inner),
            }),
            DescriptorWrite::Sampler(sampler) => self.entries.push(BindGroupEntry {
                binding,
                resource: BindingResource::Sampler(&sampler.inner),
            }),
        }

        self
    }

    pub fn build(&self, device: &Device) -> DescriptorSet {
        let group = device.create_bind_group(&BindGroupDescriptor {
            label: self.label,
            layout: &self.layout.backend,
            entries: &self.entries,
        });

        DescriptorSet { backend: group }
    }
}

pub enum DescriptorWrite<T> {
    UniformBuffer(Buffer<T>),
    TextureView(TextureView),
    Sampler(Sampler),
}

pub enum DescriptorBindingType {
    UniformBuffer,
    TextureView,
    Sampler,
    Storage { read_only: bool },
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
