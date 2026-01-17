use std::{slice, time::Instant};

use bytemuck::{Pod, Zeroable};
use maple::prelude::Config;
use maple_app::{app::App, plugin::Plugin};
use maple_engine::Scene;
use maple_renderer::{
    core::{
        PipelineCreateInfo, RenderContext, RenderPipeline, ShaderPair,
        buffer::Buffer,
        context::RenderOptions,
        descriptor_set::{
            DescriptorBindingType, DescriptorSet, DescriptorSetLayoutDescriptor, StageFlags,
        },
        texture::{SamplerOptions, Texture, TextureCreateInfo, TextureUsage},
    },
    render_graph::{
        graph::RenderGraphContext,
        node::{RenderNode, RenderTarget},
    },
    types::Vertex,
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Params {
    pub zoom: f32,
    pub aspect: f32,
    pub center: [f32; 2],
    pub max_iter: i32,
}

fn main() {
    App::new(Config::default()).add_plugin(MainPlugin).run();
}

struct MainPlugin;

impl Plugin for MainPlugin {
    fn ready(&self, app: &mut App<maple_app::app::Running>) {
        let mut graph = app.renderer_mut().graph();

        graph.add_node_with(ShowPass::setup);
        graph.add_node_with(MainPass::setup);

        graph.add_edge::<MainPass, ShowPass>();
    }
}

struct ShowPass {
    vertex_buffer: Buffer<[Vertex]>,
    pipeline: RenderPipeline,
}

impl ShowPass {
    fn setup(rcx: &RenderContext, _gcx: &mut RenderGraphContext) -> Self {
        let verticies = vec![
            Vertex {
                position: [-1.0, -1.0, 0.0],
                normal: [0.0, 0.0, -1.0],
                tex_uv: [0.0, 0.0],
                tangent: [1.0, 0.0, 0.0],
                bitangent: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [3.0, -1.0, 0.0],
                normal: [0.0, 0.0, -1.0],
                tex_uv: [2.0, 0.0],
                tangent: [1.0, 0.0, 0.0],
                bitangent: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [-1.0, 3.0, 0.0],
                normal: [0.0, 0.0, -1.0],
                tex_uv: [0.0, 2.0],
                tangent: [1.0, 0.0, 0.0],
                bitangent: [0.0, 1.0, 0.0],
            },
        ];

        let vertex_buffer = rcx.create_vertex_buffer(&verticies);

        let layout = rcx.create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
            label: Some("show"),
            visibility: StageFlags::FRAGMENT,
            layout: &[
                DescriptorBindingType::Sampler { filtering: true },
                DescriptorBindingType::TextureView { filterable: true },
            ],
        });

        let shader = rcx.create_shader_pair(ShaderPair::Glsl {
            vert: VERTEX_SHOW_SRC,
            frag: FRAG_SHOW_SRC,
        });

        let pipeline_layout = rcx.create_pipeline_layout(&[layout]);

        let pipeline = rcx.create_pipeline(PipelineCreateInfo {
            label: Some("madelbrot"),
            alpha_mode: maple_renderer::core::AlphaMode::Opaque,
            color_formats: &[rcx.surface_format()],
            cull_mode: maple_renderer::core::CullMode::None,
            layout: pipeline_layout,
            depth: &maple_renderer::render_graph::node::DepthMode::None,
            shader,
            sample_count: 1,
            use_vertex_buffer: true,
        });

        Self {
            vertex_buffer,
            pipeline,
        }
    }
}

impl RenderNode for ShowPass {
    fn draw<'a>(
        &mut self,
        rcx: &RenderContext,
        graph_ctx: &mut maple_renderer::render_graph::graph::RenderGraphContext,
        _scene: &Scene,
    ) {
        let set = graph_ctx.get_shared_resource("main/output").unwrap();

        let pipeline = &self.pipeline;

        rcx.render(
            RenderOptions {
                label: Some("Show Pass"),
                color_targets: &[RenderTarget::Surface],
                depth_target: None,
                clear_color: Some([0.0, 0.0, 0.0, 1.0]),
                clear_depth: None,
            },
            |mut fb| {
                fb.use_pipeline(pipeline)
                    .bind_vertex_buffer(&self.vertex_buffer)
                    .bind_descriptor_set(0, set)
                    .draw_vertices();
            },
        )
        .expect("failed to render show pass");
    }
}

struct MainPass {
    vertex_buffer: Buffer<[Vertex]>,
    index_buffer: Buffer<[u32]>,
    pipeline: RenderPipeline,
    target: Texture,
    params: Params,
    param_buffer: Buffer<Params>,
    descriptor_set: DescriptorSet,
    time: Instant,
}

impl MainPass {
    fn setup(rcx: &RenderContext, gcx: &mut RenderGraphContext) -> Self {
        let verticies = vec![
            Vertex {
                position: [-1.0, -1.0, 0.0],
                normal: [0.0, 0.0, -1.0],
                tex_uv: [0.0, 0.0],
                tangent: [1.0, 0.0, 0.0],
                bitangent: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [3.0, -1.0, 0.0],
                normal: [0.0, 0.0, -1.0],
                tex_uv: [2.0, 0.0],
                tangent: [1.0, 0.0, 0.0],
                bitangent: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [-1.0, 3.0, 0.0],
                normal: [0.0, 0.0, -1.0],
                tex_uv: [0.0, 2.0],
                tangent: [1.0, 0.0, 0.0],
                bitangent: [0.0, 1.0, 0.0],
            },
        ];
        let params = Params {
            zoom: 2.5,
            aspect: 1.7777,
            center: [-0.5, -0.6017],
            max_iter: 100,
        };

        let indicies: [u32; 3] = [0, 1, 2];

        let vertex_buffer = rcx.create_vertex_buffer(&verticies);
        let index_buffer = rcx.create_index_buffer(&indicies);
        let uniform_buffer = rcx.create_uniform_buffer(&params);

        let descriptor_set_layout =
            rcx.create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
                label: None,
                visibility: StageFlags::FRAGMENT,
                layout: &[DescriptorBindingType::UniformBuffer],
            });

        let descriptor_set = rcx.build_descriptor_set(
            DescriptorSet::builder(&descriptor_set_layout)
                .label("params")
                .uniform(0, &uniform_buffer),
        );

        let shader = rcx.create_shader_pair(ShaderPair::Glsl {
            vert: VERTEX_SHADER_SRC,
            frag: FRAGMENT_SHADER_SRC,
        });

        let tex = rcx.create_texture(TextureCreateInfo {
            label: None,
            width: 1920,
            height: 1080,
            format: maple_renderer::core::texture::TextureFormat::RGBA8,
            usage: TextureUsage::RENDER_ATTACHMENT | TextureUsage::TEXTURE_BINDING,
            sample_count: 1,
            mip_level: 1,
        });

        let sampler = rcx.create_sampler(SamplerOptions {
            mag_filter: maple_renderer::core::texture::FilterMode::Linear,
            min_filter: maple_renderer::core::texture::FilterMode::Linear,
            mode_u: maple_renderer::core::texture::TextureMode::Repeat,
            mode_v: maple_renderer::core::texture::TextureMode::Repeat,
            mode_w: maple_renderer::core::texture::TextureMode::Repeat,
            compare: None,
        });

        let view = tex.create_view();

        let layout = rcx.create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
            label: Some("show"),
            visibility: StageFlags::FRAGMENT,
            layout: &[
                DescriptorBindingType::Sampler { filtering: true },
                DescriptorBindingType::TextureView { filterable: true },
            ],
        });

        let set = rcx.build_descriptor_set(
            DescriptorSet::builder(&layout)
                .label("output")
                .sampler(0, &sampler)
                .texture_view(1, &view),
        );

        gcx.add_shared_resource("main/output", set);

        let pipeline = rcx.create_pipeline(PipelineCreateInfo {
            label: Some("mandlebrot"),
            layout: rcx.create_pipeline_layout(slice::from_ref(&descriptor_set_layout)),
            shader,
            color_formats: &[tex.format()],
            depth: &maple_renderer::render_graph::node::DepthMode::None,
            cull_mode: maple_renderer::core::CullMode::Back,
            alpha_mode: maple_renderer::core::AlphaMode::Opaque,
            sample_count: 1,
            use_vertex_buffer: true,
        });

        Self {
            vertex_buffer,
            index_buffer,
            param_buffer: uniform_buffer,
            target: tex,
            pipeline,
            descriptor_set,
            params,
            time: Instant::now(),
        }
    }
}

impl RenderNode for MainPass {
    fn draw(
        &mut self,
        rcx: &RenderContext,
        _graph_ctx: &mut maple_renderer::render_graph::graph::RenderGraphContext,
        _scene: &Scene,
    ) {
        let dt = self.time.elapsed().as_secs_f32();

        let fps = 1.0 / dt;

        println!("fps: {fps}");
        self.time = Instant::now();

        self.params.zoom *= 0.99_f32.powf(dt * 60.0);
        println!("zoom: {}", self.params.zoom);
        self.params.max_iter = calc_max_iter_cpu(self.params.zoom);
        print!("\x1b[2A");

        let pipeline = &self.pipeline;

        rcx.write_buffer(&self.param_buffer, &self.params);

        rcx.render(
            RenderOptions {
                label: Some("Mandlebrot"),
                color_targets: &[RenderTarget::Texture(self.target.create_view())],
                depth_target: None,
                clear_color: None,
                clear_depth: None,
            },
            |mut fb| {
                fb.use_pipeline(pipeline)
                    .debug_marker("binding verticies")
                    .bind_vertex_buffer(&self.vertex_buffer)
                    .debug_marker("binding indicies")
                    .bind_index_buffer(&self.index_buffer)
                    .debug_marker("binding descriptor")
                    .bind_descriptor_set(0, &self.descriptor_set)
                    .debug_marker("drawing")
                    .draw_indexed();
            },
        )
        .expect("failed to render mandlebrot");
    }

    fn resize(&mut self, _rcx: &RenderContext, dimensions: [u32; 2]) {
        self.params.aspect = dimensions[0] as f32 / dimensions[1] as f32;
    }
}

fn calc_max_iter_cpu(zoom: f32) -> i32 {
    let z = zoom.max(1e-12);
    let factor = 120.0 * (1.5f32).powf(-z.log2());
    factor.clamp(100.0, 2000.0) as i32
}

const VERTEX_SHOW_SRC: &str = r#"
 #version 450

 layout(location = 0) in vec3 position;   // used
 layout(location = 1) in vec3 normal;     // unused
 layout(location = 2) in vec2 tex_uv;     // unused

 layout(location = 0) out vec2 v_uv;

 void main() {
     // Your fullscreen triangle uses NDC positions like (-1,-1), (3,-1), (-1,3)
     gl_Position = vec4(position, 1.0);

     // Map NDC [-1,1] -> UV [0,1]; works even for the oversized FS triangle
     v_uv = gl_Position.xy * 0.5 + 0.5;
 }
 "#;

const FRAG_SHOW_SRC: &str = r#"
#version 450

layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 frag_out;

layout(set = 0, binding = 0) uniform sampler show_sampler;
layout(set = 0, binding = 1) uniform texture2D show_tex;

void main() {
    frag_out = texture(sampler2D(show_tex, show_sampler), v_uv);
}
 "#;

const VERTEX_SHADER_SRC: &str = r#"
 #version 450

 layout(location = 0) in vec3 position;   // used
 layout(location = 1) in vec3 normal;     // unused
 layout(location = 2) in vec2 tex_uv;     // unused

 layout(location = 0) out vec2 v_uv;

 void main() {
     // Your fullscreen triangle uses NDC positions like (-1,-1), (3,-1), (-1,3)
     gl_Position = vec4(position, 1.0);

     // Map NDC [-1,1] -> UV [0,1]; works even for the oversized FS triangle
     v_uv = gl_Position.xy * 0.5 + 0.5;
 }
 "#;

const FRAGMENT_SHADER_SRC: &str = r#"
#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 out_color;
layout(set = 0, binding = 0) uniform Params {
    float zoom;
    float aspect;
    vec2  center;
} params;

// Double-single arithmetic helpers
vec2 ds_add(vec2 a, float b) {
    float t1 = a.x + b;
    float t2 = b - (t1 - a.x);
    return vec2(t1, a.y + t2);
}

vec2 ds_add(vec2 a, vec2 b) {
    float t1 = a.x + b.x;
    float t2 = b.x - (t1 - a.x);
    float t3 = a.y + b.y + t2;
    return vec2(t1, t3);
}

vec2 ds_mul(vec2 a, vec2 b) {
    float c = a.x * b.x;
    float c_hi = c;
    float c_lo = fma(a.x, b.x, -c) + a.x * b.y + a.y * b.x;
    return vec2(c_hi, c_lo);
}

vec2 ds_mul(vec2 a, float b) {
    float c = a.x * b;
    float c_hi = c;
    float c_lo = fma(a.x, b, -c) + a.y * b;
    return vec2(c_hi, c_lo);
}

// z = z^2 + c in double-single precision
void ds_mandel_iter(inout vec2 zx, inout vec2 zy, vec2 cx, vec2 cy) {
    vec2 zx2 = ds_mul(zx, zx);
    vec2 zy2 = ds_mul(zy, zy);
    vec2 zxzy = ds_mul(zx, zy);
    
    zy = ds_add(ds_add(zxzy, zxzy), cy);
    zx = ds_add(ds_add(zx2, vec2(-zy2.x, -zy2.y)), cx);
}

float distanceToMandelbrot(vec2 cx, vec2 cy, int max_iter) {
    // Quick bulb checks with single precision (use high component)
    vec2 c_f = vec2(cx.x, cy.x);
    float c2 = dot(c_f, c_f);
    if (256.0*c2*c2 - 96.0*c2 + 32.0*c_f.x - 3.0 < 0.0) return 0.0;
    if (16.0*(c2 + 2.0*c_f.x + 1.0) - 1.0 < 0.0) return 0.0;
    
    vec2 zx = vec2(0.0);
    vec2 zy = vec2(0.0);
    vec2 dzx = vec2(1.0, 0.0);  // Start dz at 1
    vec2 dzy = vec2(0.0);
    
    float m2 = 0.0;
    
    for (int i = 0; i < max_iter; ++i) {
        if (m2 > 1024.0) break;
        
        // dz = 2 * z * dz + 1
        vec2 temp_x = ds_mul(zx, dzx);
        vec2 temp_y = ds_mul(zy, dzy);
        vec2 temp_x2 = ds_mul(zx, dzy);
        vec2 temp_y2 = ds_mul(zy, dzx);
        
        vec2 new_dzx = ds_add(temp_x, vec2(-temp_y.x, -temp_y.y));
        vec2 new_dzy = ds_add(temp_x2, temp_y2);
        
        dzx = ds_add(ds_add(new_dzx, new_dzx), vec2(1.0, 0.0));
        dzy = ds_add(new_dzy, new_dzy);
        
        // z = z^2 + c
        ds_mandel_iter(zx, zy, cx, cy);
        
        m2 = zx.x * zx.x + zy.x * zy.x;
    }
    
    if (m2 <= 1024.0) return 0.0;
    
    float r2 = max(m2, 1e-24);
    float dz2 = max(dzx.x*dzx.x + dzy.x*dzy.x, 1e-24);
    return 0.5 * sqrt(r2 / dz2) * log(r2);
}

int calc_max_iter(float zoom) {
    float factor = 120.0 * pow(1.5, -log2(max(zoom, 1e-12)));
    factor = clamp(factor, 100.0, 2000.0);
    return int(floor(factor));
}

void main() {
    float zoom = max(params.zoom, 1e-12);
    int max_iter = calc_max_iter(zoom);
    
    // Map screen uv -> complex plane
    vec2 uv = v_uv * 2.0 - 1.0; // [-1,1]
    uv.x *= params.aspect;
    
    vec2 center_x = vec2(params.center.x, 0.0);
    vec2 center_y = vec2(params.center.y, 0.0);
    
    vec2 offset_x = ds_mul(vec2(uv.x, 0.0), zoom);
    vec2 offset_y = ds_mul(vec2(uv.y, 0.0), zoom);
    
    vec2 cx = ds_add(center_x, offset_x);
    vec2 cy = ds_add(center_y, offset_y);
    
    // Distance to set
    float d = distanceToMandelbrot(cx, cy, max_iter);
    
    // Soft shading
    float t = clamp(pow(4.0 * d / zoom, 0.2), 0.0, 1.0);
    
    vec3 color = mix(vec3(0.0),
                     vec3(0.0, 0.4 + 0.6 * t, 1.0),
                     t);
    
    out_color = vec4(color, 1.0);
} "#;
