use maple_app::{app::App, plugin::Plugin};
use maple_renderer::{
    backend::vulkan::buffer::data_buffer::VulkanBuffer,
    core::{
        buffer::Buffer,
        render_pass::{RenderPass, RenderPassDescriptor},
        shader::GraphicsShader,
    },
    types::{Vertex, vertex::Params},
    vulkano::{
        buffer::BufferContents,
        command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer},
        pipeline::graphics::vertex_input::VertexBuffersCollection,
    },
};

fn main() {
    App::new().add_plugin(MainPlugin).run();
}

struct MainPlugin;

impl Plugin for MainPlugin {
    fn init(&self, app: &mut App<maple_app::app::Initialized>) {
        app.add_renderpass(MainPass {
            vertex_buffer: None,
            index_buffer: None,
        });
    }
}

struct MainPass {
    vertex_buffer: Option<Buffer<[Vertex]>>,
    index_buffer: Option<Buffer<[u32]>>,
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

        let indices: Vec<u32> = vec![0, 1, 2];

        let vertex_buffer = renderer.create_vertex_buffer(verticies).unwrap();
        let index_buffer = renderer.create_index_buffer(indices).unwrap();

        let params = Params {
            center_zoom_aspect: [-0.5, 0.0, 2.5, 1.7777],
            iter_pad: [1000.0, 0.0, 0.0, 0.0],
        };

        self.vertex_buffer = Some(vertex_buffer);
        self.index_buffer = Some(index_buffer);

        let shader = renderer
            .create_shader_from_slice(VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC)
            .unwrap();

        RenderPassDescriptor {
            name: "main",
            shader,
            format: None,
            depth_format: None,
            viewport: None,
        }
    }

    fn draw(
        &mut self,
        // TODO abstract the command buffer into api agnostic builder
        command_buffer_builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        drawables: &[&dyn maple_renderer::types::drawable::Drawable],
    ) -> anyhow::Result<()> {
        unsafe {
            let vertex_buffer = (*self.vertex_buffer.as_ref().unwrap()).clone();
            let index_buffer = (*self.index_buffer.as_ref().unwrap()).clone();

            command_buffer_builder
                .bind_vertex_buffers(0, VulkanBuffer::from(vertex_buffer))?
                .bind_index_buffer(VulkanBuffer::from(index_buffer))?
                .draw_indexed(3, 1, 0, 0, 0)?;
        }

        Ok(())
    }
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

// Mandelbrot configuration (same spirit as your compute shader)
const float max_iter = 1000.0;
const float zoom     = 0.01;
const vec2  center   = vec2(-0.95, 0.25);

// If you want correct aspect (no stretch), change this to width/height via a uniform/push constant.
const float aspect   = 1.7777;

void main() {
    // Map to complex plane
    vec2 uv = v_uv * 2.0 - 1.0; // [-1,1]
    uv.x *= aspect;             // fix stretching if you provide aspect

    vec2 c = uv * zoom + center;

    vec2 z = vec2(0.0);
    float i;
    for (i = 0.0; i < max_iter; i += 1.0) {
        // z = z^2 + c
        vec2 z2 = vec2(
            z.x*z.x - z.y*z.y + c.x,
            2.0*z.x*z.y + c.y
        );
        z = z2;

        if (dot(z, z) > 16.0) break; // 4^2 avoids a sqrt
    }

    // Smooth coloring
    float len = length(z);
    float smooth_i = i - log2(log(max(len, 1e-8))) + 4.0;
    float t = clamp(smooth_i / max_iter, 0.0, 1.0);

    // Blue outside, black inside
    vec3 color = mix(vec3(0.0, 0.0, 0.0),
                     vec3(0.0, 0.4 + 0.6 * t, 1.0),
                     t);

    out_color = vec4(color, 1.0);
}
"#;
