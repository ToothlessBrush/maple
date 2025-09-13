use std::time::Instant;

use bytemuck::{Pod, Zeroable};
use maple::prelude::Config;
use maple_app::{app::App, plugin::Plugin};
use maple_renderer::{
    core::{
        RenderContext, ShaderPair,
        buffer::Buffer,
        descriptor_set::{
            DescriptorBindingType, DescriptorSet, DescriptorSetLayout,
            DescriptorSetLayoutDescriptor, StageFlags,
        },
        texture::{SamplerOptions, TextureCreateInfo, TextureUsage},
    },
    render_graph::{
        graph::RenderGraphContext,
        node::{RenderNode, RenderNodeDescriptor, RenderTarget},
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
    fn init(&self, app: &mut App<maple_app::app::Running>) {
        let mut graph = app.renderer().graph();

        graph.add_node(
            ShowPass::SHOW,
            ShowPass {
                vertex_buffer: None,
            },
        );

        graph.add_node(
            MainPass::MAIN,
            MainPass {
                vertex_buffer: None,
                index_buffer: None,
                params: Params {
                    zoom: 2.5,
                    aspect: 1.7777,
                    center: [-0.5, -0.6017],
                    max_iter: 100,
                },
                descriptor_layout: None,
                descriptor_set: None,
                param_buffer: None,
                time: Instant::now(),
            },
        );

        graph.add_edge(ShowPass::SHOW, MainPass::MAIN);
    }
}

struct ShowPass {
    vertex_buffer: Option<Buffer<[Vertex]>>,
}

impl ShowPass {
    const SHOW: &str = "show";
}
impl RenderNode for ShowPass {
    fn setup(
        &mut self,
        render_ctx: &RenderContext,
        graph_ctx: &mut maple_renderer::render_graph::graph::RenderGraphContext,
    ) -> RenderNodeDescriptor {
        let verticies = vec![
            Vertex {
                position: [-1.0, -1.0, 0.0],
                normal: [0.0, 0.0, -1.0],
                tex_uv: [0.0, 0.0],
            },
            Vertex {
                position: [3.0, -1.0, 0.0],
                normal: [0.0, 0.0, -1.0],
                tex_uv: [2.0, 0.0],
            },
            Vertex {
                position: [-1.0, 3.0, 0.0],
                normal: [0.0, 0.0, -1.0],
                tex_uv: [0.0, 2.0],
            },
        ];

        let vertex_buffer = render_ctx.create_vertex_buffer(&verticies);
        self.vertex_buffer = Some(vertex_buffer);

        let layout = render_ctx.create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
            label: Some("show"),
            visibility: StageFlags::FRAGMENT,
            layout: &[
                DescriptorBindingType::Sampler,
                DescriptorBindingType::TextureView,
            ],
        });

        let shader = render_ctx.create_shader_pair(ShaderPair::Glsl {
            vert: VERTEX_SHOW_SRC,
            frag: FRAG_SHOW_SRC,
        });

        RenderNodeDescriptor {
            shader,
            descriptor_set_layouts: vec![layout],
            target: RenderTarget::Surface,
        }
    }

    fn draw<'a>(
        &mut self,
        render_ctx: &RenderContext,
        node_ctx: &mut maple_renderer::render_graph::node::RenderNodeContext,
        graph_ctx: &mut maple_renderer::render_graph::graph::RenderGraphContext,
        world: maple_renderer::types::world::World<'a>,
    ) -> anyhow::Result<()> {
        let set = graph_ctx.get_shared_resource("main/output").unwrap();

        render_ctx.render(&node_ctx, |mut fb| {
            fb.bind_vertex_buffer(self.vertex_buffer.as_ref().unwrap())
                .bind_descriptor_set(0, set)
                .draw();
        })
    }
}

struct MainPass {
    vertex_buffer: Option<Buffer<[Vertex]>>,
    index_buffer: Option<Buffer<[u32]>>,
    params: Params,
    param_buffer: Option<Buffer<Params>>,
    descriptor_layout: Option<DescriptorSetLayout>,
    descriptor_set: Option<DescriptorSet>,
    time: Instant,
}

impl MainPass {
    const MAIN: &str = "main pass";
}

impl RenderNode for MainPass {
    fn setup(&mut self, rcx: &RenderContext, gcx: &mut RenderGraphContext) -> RenderNodeDescriptor {
        let verticies = vec![
            Vertex {
                position: [-1.0, -1.0, 0.0],
                normal: [0.0, 0.0, -1.0],
                tex_uv: [0.0, 0.0],
            },
            Vertex {
                position: [3.0, -1.0, 0.0],
                normal: [0.0, 0.0, -1.0],
                tex_uv: [2.0, 0.0],
            },
            Vertex {
                position: [-1.0, 3.0, 0.0],
                normal: [0.0, 0.0, -1.0],
                tex_uv: [0.0, 2.0],
            },
        ];

        let indicies: [u32; 3] = [0, 1, 2];

        let vertex_buffer = rcx.create_vertex_buffer(&verticies);
        let index_buffer = rcx.create_index_buffer(&indicies);
        let uniform_buffer = rcx.create_uniform_buffer(&self.params);

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
        });

        let sampler = rcx.create_sampler(SamplerOptions {
            mag_filter: maple_renderer::core::texture::FilterMode::Linear,
            min_filter: maple_renderer::core::texture::FilterMode::Linear,
            mode_u: maple_renderer::core::texture::TextureMode::Repeat,
            mode_v: maple_renderer::core::texture::TextureMode::Repeat,
            mode_w: maple_renderer::core::texture::TextureMode::Repeat,
        });

        let view = tex.create_view();

        let layout = rcx.create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
            label: Some("show"),
            visibility: StageFlags::FRAGMENT,
            layout: &[
                DescriptorBindingType::Sampler,
                DescriptorBindingType::TextureView,
            ],
        });

        let set = rcx.build_descriptor_set(
            DescriptorSet::builder(&layout)
                .label("output")
                .sampler(0, &sampler)
                .texture_view(1, &view),
        );

        gcx.add_shared_resource("main/output", set);

        self.vertex_buffer = Some(vertex_buffer);
        self.index_buffer = Some(index_buffer);
        self.param_buffer = Some(uniform_buffer);

        self.descriptor_layout = Some(descriptor_set_layout.clone());
        self.descriptor_set = Some(descriptor_set);

        RenderNodeDescriptor {
            shader,
            descriptor_set_layouts: vec![descriptor_set_layout],
            target: RenderTarget::Texture(tex),
        }
    }

    fn draw<'a>(
        &mut self,
        render_ctx: &RenderContext,
        node_ctx: &mut maple_renderer::render_graph::node::RenderNodeContext,
        graph_ctx: &mut maple_renderer::render_graph::graph::RenderGraphContext,
        world: maple_renderer::types::world::World<'a>,
    ) -> anyhow::Result<()> {
        let fps = 1.0 / self.time.elapsed().as_secs_f64();

        println!("fps: {fps}");
        self.time = Instant::now();

        self.params.zoom *= 0.999;
        self.params.max_iter = calc_max_iter_cpu(self.params.zoom);

        render_ctx.write_buffer(self.param_buffer.as_ref().unwrap(), &self.params)?;

        render_ctx.render(node_ctx, |mut fb| {
            fb.debug_marker("binding verticies")
                .bind_vertex_buffer(self.vertex_buffer.as_ref().unwrap())
                .debug_marker("binding indicies")
                .bind_index_buffer(self.index_buffer.as_ref().unwrap())
                .debug_marker("binding descriptor")
                .bind_descriptor_set(0, self.descriptor_set.as_ref().unwrap())
                .debug_marker("drawing")
                .draw_indexed();
        })?;

        Ok(())
    }

    fn resize(&mut self, dimensions: [u32; 2]) -> anyhow::Result<()> {
        self.params.aspect = dimensions[0] as f32 / dimensions[1] as f32;

        Ok(())
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

// Uniforms: keep your existing layout
layout(set = 0, binding = 0) uniform Params {
    float zoom;
    float aspect;   // keep for std140 alignment
    vec2  center;
} params;

// Zoom-adaptive max-iteration (tuned for DE)
int calc_max_iter(float zoom) {
    // Similar to your original idea, but keep floors sane for DE
    float factor = 120.0 * pow(1.5, -log2(max(zoom, 1e-12)));
    factor = clamp(factor, 100.0, 2000.0);
    return int(floor(factor));
}

// Distance estimator to the Mandelbrot set.
// Returns 0.0 if it didn't escape (likely inside), otherwise the DE distance.
float distanceToMandelbrot(vec2 c, int max_iter)
{
    // Quick bulbs (inside tests) from iq
    {
        float c2 = dot(c, c);
        // cardioid / main bulb (M1)
        if (256.0*c2*c2 - 96.0*c2 + 32.0*c.x - 3.0 < 0.0) return 0.0;
        // period-2 bulb (M2)
        if (16.0*(c2 + 2.0*c.x + 1.0) - 1.0 < 0.0) return 0.0;
    }

    vec2  z  = vec2(0.0);
    vec2  dz = vec2(0.0);
    float m2 = 0.0;
    float escaped = 0.0; // 0 = escaped, 1 = did not escape (matches iq’s di usage inverted below)

    for (int i = 0; i < max_iter; ++i) {
        // Escape check (radius 32, i.e., m2 > 1024) — generous for DE stability
        if (m2 > 1024.0) { escaped = 1.0; break; }

        // dz' = 2 * z * dz + 1  (complex derivative)
        dz = 2.0 * vec2(z.x*dz.x - z.y*dz.y, z.x*dz.y + z.y*dz.x) + vec2(1.0, 0.0);

        // z = z^2 + c
        z  = vec2(z.x*z.x - z.y*z.y, 2.0*z.x*z.y) + c;

        m2 = dot(z, z);
    }

    if (escaped < 0.5) {
        // Didn't escape within max_iter: treat as inside → DE = 0
        return 0.0;
    }

    // DE formula: d(c) ≈ 0.5 * |z| * log|z| / |dz|
    float r2 = max(m2, 1e-24);                 // |z|^2
    float dz2 = max(dot(dz, dz), 1e-24);       // |dz|^2
    float d = 0.5 * sqrt(r2 / dz2) * log(r2);  // note: log(|z|^2) = 2*log|z|; factor absorbed by 0.5

    return max(d, 0.0);
}

void main() {
    float zoom   = max(params.zoom, 1e-12);
    vec2  center = params.center;

    int max_iter = calc_max_iter(zoom);

    // Map screen uv -> complex plane (keeping your aspect handling)
    vec2 uv = v_uv * 2.0 - 1.0; // [-1,1]
    uv.x *= aspect;

    vec2 c = center + uv * zoom;

    // Distance to set
    float d = distanceToMandelbrot(c, max_iter);

    // Soft shading from iq: scale by zoom to keep perceived thickness stable while zooming
    float t = clamp(pow(4.0 * d / zoom, 0.2), 0.0, 1.0);

    // Keep your blue ramp for exterior; solid black for interior (t==0)
    vec3 color = mix(vec3(0.0),
                     vec3(0.0, 0.4 + 0.6 * t, 1.0),
                     t);

    out_color = vec4(color, 1.0);
}
 "#;
