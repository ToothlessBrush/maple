use bytemuck::{AnyBitPattern, Pod, Zeroable};
use maple_app::{app::App, plugin::Plugin};
use maple_renderer::{
    core::{
        ShaderPair,
        buffer::Buffer,
        descriptor_set::{
            DescriptorBindingDesc, DescriptorBindingType, DescriptorSet, DescriptorSetDescriptor,
            DescriptorSetLayout, DescriptorSetLayoutDescriptor, DescriptorWrite, StageFlags,
        },
        pipeline::RenderPipeline,
        render_pass::{RenderPass, RenderPassDescriptor},
        renderer::Renderer,
        shader::GraphicsShader,
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
    App::new().add_plugin(MainPlugin).run();
}

struct MainPlugin;

impl Plugin for MainPlugin {
    fn init(&self, app: &mut App<maple_app::app::Running>) {
        app.add_renderpass(MainPass {
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
        });
    }
}

struct MainPass {
    vertex_buffer: Option<Buffer<[Vertex]>>,
    index_buffer: Option<Buffer<[u32]>>,
    params: Params,
    param_buffer: Option<Buffer<Params>>,
    descriptor_layout: Option<DescriptorSetLayout>,
    descriptor_set: Option<DescriptorSet>,
}

impl RenderPass for MainPass {
    fn setup(
        &mut self,
        renderer: &maple_renderer::core::renderer::Renderer,
    ) -> RenderPassDescriptor {
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

        let vertex_buffer = renderer.create_vertex_buffer(&verticies);
        let index_buffer = renderer.create_index_buffer(&indicies);
        let uniform_buffer = renderer.create_uniform_buffer(&self.params);

        let descriptor_set_layout =
            renderer.create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
                label: None,
                visibility: StageFlags::FRAGMENT,
                layout: &[DescriptorBindingType::UniformBuffer],
            });

        let descriptor_set = renderer.create_descriptor_set(DescriptorSetDescriptor {
            label: None,
            layout: &descriptor_set_layout,
            writes: &[DescriptorWrite::UniformBuffer {
                binding: 0,
                buffer: uniform_buffer.clone(),
            }],
        });

        let shader = renderer.create_shader_pair(ShaderPair::Glsl {
            vert: VERTEX_SHADER_SRC,
            frag: FRAGMENT_SHADER_SRC,
        });

        self.vertex_buffer = Some(vertex_buffer);
        self.index_buffer = Some(index_buffer);
        self.param_buffer = Some(uniform_buffer);

        self.descriptor_layout = Some(descriptor_set_layout.clone());
        self.descriptor_set = Some(descriptor_set);

        RenderPassDescriptor {
            name: "main pass",
            shader,
            descriptor_set_layouts: vec![descriptor_set_layout],
        }
    }

    fn draw(
        &mut self,
        renderer: &Renderer,
        pipeline: &RenderPipeline,
        drawables: &[&dyn maple_renderer::types::drawable::Drawable],
    ) -> anyhow::Result<()> {
        self.params.zoom *= 0.999;
        self.params.max_iter = calc_max_iter_cpu(self.params.zoom);

        renderer.write_buffer(self.param_buffer.as_ref().unwrap(), &self.params)?;

        renderer.render(pipeline, |mut fb| {
            fb.debug_marker("binding verticies")
                .bind_vertex_buffer(self.vertex_buffer.as_ref().unwrap())
                .debug_marker("binding indicies")
                .bind_index_buffer(self.index_buffer.as_ref().unwrap())
                .debug_marker("binding descriptor")
                .bind_descriptor_set(0, self.descriptor_set.as_ref().unwrap())
                .debug_marker("drawing")
                .draw_indexed();
        });

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
