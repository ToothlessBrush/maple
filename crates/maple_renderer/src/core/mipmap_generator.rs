use std::{collections::HashMap, sync::OnceLock};
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, CommandEncoder, ComputePipeline,
    ComputePipelineDescriptor, Device, PipelineLayoutDescriptor, Queue, SamplerBindingType,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, StorageTextureAccess, TextureFormat,
    TextureSampleType, TextureViewDescriptor, TextureViewDimension,
};

pub struct MipmapGenerator {
    pipelines: HashMap<TextureFormat, ComputePipeline>,
    bind_group_layouts: HashMap<TextureFormat, BindGroupLayout>,
    filtering_sampler: wgpu::Sampler,
    non_filtering_sampler: wgpu::Sampler,
    format_is_filterable: HashMap<TextureFormat, bool>,
}

impl MipmapGenerator {
    pub fn new(device: &Device) -> Self {
        let filtering_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Mipmap Generator Filtering Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let non_filtering_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Mipmap Generator Non-Filtering Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // List of formats that support storage textures for mipmap generation
        // Format: (wgpu format, WGSL format string, is_filterable)
        // Note: sRGB formats (Rgba8UnormSrgb, etc.) do NOT support storage textures in WebGPU
        // Rgba32Float is not hardware filterable, so we use manual filtering
        let supported_formats = [
            (TextureFormat::Rgba8Unorm, "rgba8unorm", true),
            (TextureFormat::Rgba8Snorm, "rgba8snorm", true),
            (TextureFormat::Rgba16Float, "rgba16float", true),
            (TextureFormat::Rgba32Float, "rgba32float", false),
        ];

        let mut pipelines = HashMap::new();
        let mut bind_group_layouts = HashMap::new();
        let mut format_is_filterable = HashMap::new();

        // Load the shader template
        let shader_template = include_str!("mipmap_generator.wgsl");

        for (format, wgsl_format, is_filterable) in supported_formats {
            format_is_filterable.insert(format, is_filterable);
            // Generate shader source for this specific format
            let shader_source = shader_template.replace("rgba8unorm", wgsl_format);

            let shader = device.create_shader_module(ShaderModuleDescriptor {
                label: Some("Mipmap Generator Shader"),
                source: ShaderSource::Wgsl(shader_source.into()),
            });

            let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Mipmap Generator Bind Group Layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: is_filterable },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::WriteOnly,
                            format,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Sampler(
                            if is_filterable {
                                SamplerBindingType::Filtering
                            } else {
                                SamplerBindingType::NonFiltering
                            }
                        ),
                        count: None,
                    },
                ],
            });

            let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Mipmap Generator Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

            let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
                label: Some("Mipmap Generator Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            });

            bind_group_layouts.insert(format, bind_group_layout);
            pipelines.insert(format, pipeline);
        }

        Self {
            pipelines,
            bind_group_layouts,
            filtering_sampler,
            non_filtering_sampler,
            format_is_filterable,
        }
    }

    fn generate_with_encoder(
        &self,
        device: &Device,
        encoder: &mut CommandEncoder,
        texture: &wgpu::Texture,
        mip_level_count: u32,
    ) {
        let format = texture.format();

        // Check if we have a pipeline for this format
        let (pipeline, bind_group_layout) = if let (Some(pipeline), Some(layout)) =
            (self.pipelines.get(&format), self.bind_group_layouts.get(&format)) {
            (pipeline, layout)
        } else {
            log::warn!(
                "Mipmap generation not supported for format {:?}. Skipping mipmap generation.",
                format
            );
            return;
        };

        for mip_level in 1..mip_level_count {
            let src_view = texture.create_view(&TextureViewDescriptor {
                label: Some("Mipmap Src View"),
                base_mip_level: mip_level - 1,
                mip_level_count: Some(1),
                ..Default::default()
            });

            let dst_view = texture.create_view(&TextureViewDescriptor {
                label: Some("Mipmap Dst View"),
                base_mip_level: mip_level,
                mip_level_count: Some(1),
                ..Default::default()
            });

            // Select the correct sampler based on format filterability
            let is_filterable = self.format_is_filterable.get(&format).copied().unwrap_or(true);
            let sampler = if is_filterable {
                &self.filtering_sampler
            } else {
                &self.non_filtering_sampler
            };

            let bind_group = device.create_bind_group(&BindGroupDescriptor {
                label: Some("Mipmap Generator Bind Group"),
                layout: bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&src_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(&dst_view),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::Sampler(sampler),
                    },
                ],
            });

            let mip_width = (texture.width() >> mip_level).max(1);
            let mip_height = (texture.height() >> mip_level).max(1);

            let workgroup_size = 8;
            let dispatch_x = (mip_width + workgroup_size - 1) / workgroup_size;
            let dispatch_y = (mip_height + workgroup_size - 1) / workgroup_size;

            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Mipmap Generation Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups(dispatch_x, dispatch_y, 1);
        }
    }
}

static MIPMAP_GENERATOR: OnceLock<MipmapGenerator> = OnceLock::new();

fn get_generator(device: &Device) -> &'static MipmapGenerator {
    MIPMAP_GENERATOR.get_or_init(|| MipmapGenerator::new(device))
}

/// Generate mipmaps for a 2D texture
pub fn generate_mipmaps(
    device: &Device,
    queue: &Queue,
    texture: &wgpu::Texture,
    mip_level_count: u32,
) {
    if mip_level_count <= 1 {
        return;
    }

    let generator = get_generator(device);

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Mipmap Generation"),
    });

    generator.generate_with_encoder(device, &mut encoder, texture, mip_level_count);

    queue.submit(std::iter::once(encoder.finish()));
}
