use std::time::Instant;

use bytemuck::{Pod, Zeroable};
use maple::prelude::Config;
use maple_app::{app::App, plugin::Plugin};
use maple_renderer::core::RenderContext;
use maple_renderer::core::texture::{SamplerOptions, TextureCreateInfo, TextureUsage};
use maple_renderer::render_graph::graph::RenderGraphContext;
use maple_renderer::render_graph::node::RenderNodeDescriptor;
use maple_renderer::render_graph::node::{RenderNodeContext, RenderTarget};
use maple_renderer::types::Vertex;
use maple_renderer::types::world::{self, World};
use maple_renderer::{
    core::{
        buffer::Buffer,
        descriptor_set::{
            DescriptorBindingType, DescriptorSet, DescriptorSetDescriptor, DescriptorSetLayout,
            DescriptorSetLayoutDescriptor, StageFlags,
        },
        pipeline::RenderPipeline,
        renderer::Renderer,
        shader::{GraphicsShader, ShaderPair},
    },
    render_graph::node::RenderNode,
};

// ─────────────────────────────────────────────────────────────────────────────
// Shadertoy-style UBO (vec4-only: safe for std140 across Rust/GLSL)
//   iResolution = (width, height, pixelAspect, _)
//   iTimeData   = (iTime, iTimeDelta, iFrame_as_float, _)
//   iMouse      = (x, y, clickX, clickY)
// ─────────────────────────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct ShadertoyParams {
    pub iResolution: [f32; 4],
    pub iTimeData: [f32; 4],
    pub iMouse: [f32; 4],
}

impl Default for ShadertoyParams {
    fn default() -> Self {
        Self {
            iResolution: [1920.0, 1080.0, 1920.0 / 1080.0, 0.0],
            iTimeData: [0.0, 0.0, 0.0, 0.0],
            iMouse: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// App entry
// ─────────────────────────────────────────────────────────────────────────────

fn main() {
    App::new(Config::default()).add_plugin(ShaderToy).run();
}

struct ShaderToy;

impl Plugin for ShaderToy {
    fn init(&self, app: &mut App<maple_app::app::Running>) {
        let mut graph = app.renderer_mut().graph();

        graph.add_node("main pass", MainPass::new());

        graph.add_node(
            "composite",
            Composite {
                vertex_buffer: None,
            },
        );

        graph.add_edge("main pass", "composite");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Pass
// ─────────────────────────────────────────────────────────────────────────────

struct Composite {
    vertex_buffer: Option<Buffer<[Vertex]>>,
}

impl RenderNode for Composite {
    fn setup(
        &mut self,
        render_ctx: &RenderContext,
        graph_ctx: &mut RenderGraphContext,
    ) -> RenderNodeDescriptor {
        let vertices = vec![
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

        let vertex_buffer = render_ctx.create_vertex_buffer(&vertices);

        self.vertex_buffer = Some(vertex_buffer);

        let shader = render_ctx.create_shader_pair(ShaderPair::Glsl {
            vert: VERTEX_COMPOSITE_SRC,
            frag: FRAG_COMPOSITE_SRC,
        });

        let layout = render_ctx.create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
            label: Some("iChannel0"),
            visibility: StageFlags::FRAGMENT,
            layout: &[
                DescriptorBindingType::Sampler,
                DescriptorBindingType::TextureView,
            ],
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
        /// get the output of the last pass
        let set = graph_ctx.get_shared_resource("main/output").unwrap();

        render_ctx.render(&node_ctx, |mut fb| {
            fb.bind_vertex_buffer(self.vertex_buffer.as_ref().unwrap())
                .bind_descriptor_set(0, set)
                .draw();
        })
    }
}

struct MainPass {
    // gpu resources
    vertex_buffer: Option<Buffer<[Vertex]>>,
    index_buffer: Option<Buffer<[u32]>>,
    params_buffer: Option<Buffer<ShadertoyParams>>,
    params_layout: Option<DescriptorSetLayout>,
    params_set: Option<DescriptorSet>,

    // cpu state
    params: ShadertoyParams,
    last_instant: Instant,
    frame_counter: u32,
}

impl MainPass {
    fn new() -> Self {
        Self {
            vertex_buffer: None,
            index_buffer: None,
            params_buffer: None,
            params_layout: None,
            params_set: None,
            params: ShadertoyParams::default(),
            last_instant: Instant::now(), // Instant doesn't implement Default
            frame_counter: 0,
        }
    }
}

impl RenderNode for MainPass {
    fn setup(&mut self, rcx: &RenderContext, gcx: &mut RenderGraphContext) -> RenderNodeDescriptor {
        // Fullscreen triangle
        let vertices = vec![
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
        let indices: [u32; 3] = [0, 1, 2];

        let vbuf = rcx.create_vertex_buffer(&vertices);
        let ibuf = rcx.create_index_buffer(&indices);
        let pbuf = rcx.create_uniform_buffer(&self.params);

        // Descriptor set 0: UBO
        let layout = rcx.create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
            label: Some("shadertoy_params_layout"),
            visibility: StageFlags::FRAGMENT,
            layout: &[DescriptorBindingType::UniformBuffer],
        });

        let set = rcx.build_descriptor_set(
            DescriptorSet::builder(&layout)
                .label("shadertoy_params_set")
                .uniform(0, &pbuf),
        );

        // Shaders
        let shader = rcx.create_shader_pair(ShaderPair::Glsl {
            vert: VERT_SRC,
            frag: FRAG_SRC,
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

        let shared_layout = rcx.create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
            label: Some("show"),
            visibility: StageFlags::FRAGMENT,
            layout: &[
                DescriptorBindingType::Sampler,
                DescriptorBindingType::TextureView,
            ],
        });

        let shared_set = rcx.build_descriptor_set(
            DescriptorSet::builder(&shared_layout)
                .label("output")
                .sampler(0, &sampler)
                .texture_view(1, &view),
        );

        gcx.add_shared_resource("main/output", shared_set);

        // Store resources
        self.vertex_buffer = Some(vbuf);
        self.index_buffer = Some(ibuf);
        self.params_buffer = Some(pbuf);
        self.params_layout = Some(layout.clone());
        self.params_set = Some(set);

        RenderNodeDescriptor {
            shader,
            descriptor_set_layouts: vec![layout],
            target: RenderTarget::Texture(tex),
        }
    }

    fn draw(
        &mut self,
        rcx: &RenderContext,
        ncx: &mut RenderNodeContext,
        gcx: &mut RenderGraphContext,
        world: World,
    ) -> anyhow::Result<()> {
        // Update timing
        let now = Instant::now();
        let dt = now.duration_since(self.last_instant).as_secs_f32();

        let fps = 1.0 / dt;

        println!("fps: {fps}");

        self.last_instant = now;

        // iTimeData = (time, dt, frame_as_float, _)
        self.params.iTimeData[1] = dt.max(0.0); // dt
        self.params.iTimeData[0] += self.params.iTimeData[1]; // time += dt
        self.frame_counter = self.frame_counter.wrapping_add(1);
        self.params.iTimeData[2] = self.frame_counter as f32; // frame

        // (Optional) update mouse here if you track input:
        // self.params.iMouse = [mouse_x, mouse_y, click_x, click_y];

        // Write UBO
        rcx.write_buffer(self.params_buffer.as_ref().unwrap(), &self.params)?;

        // Draw
        rcx.render(ncx, |mut fb| {
            fb.bind_vertex_buffer(self.vertex_buffer.as_ref().unwrap())
                .bind_index_buffer(self.index_buffer.as_ref().unwrap())
                .bind_descriptor_set(0, self.params_set.as_ref().unwrap())
                .draw_indexed();
        });

        Ok(())
    }

    fn resize(&mut self, dimensions: [u32; 2]) -> anyhow::Result<()> {
        let (w, h) = (dimensions[0].max(1) as f32, dimensions[1].max(1) as f32);
        self.params.iResolution = [w, h, w / h, 0.0];
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Shaders
//   FRAG_SRC is a minimal Shadertoy-style demo.
//   To use a Shadertoy you like: keep the UBO header + main(); replace mainImage body.
// ─────────────────────────────────────────────────────────────────────────────
const VERTEX_COMPOSITE_SRC: &str = r#"
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

const FRAG_COMPOSITE_SRC: &str = r#"
#version 450

layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 frag_out;

layout(set = 0, binding = 0) uniform sampler show_sampler;
layout(set = 0, binding = 1) uniform texture2D show_tex;

void main() {
    vec3 col = texture(sampler2D(show_tex, show_sampler), v_uv).xyz;

    col *= 0.5 + 0.5*pow( 16.0*v_uv.x*v_uv.y*(1.0-v_uv.x)*(1.0-v_uv.y), 0.05 );

    frag_out = vec4(col, 1.0);
}
 "#;

const VERT_SRC: &str = r#"#version 450
layout(location=0) in vec3 position;
layout(location=1) in vec3 normal;
layout(location=2) in vec2 tex_uv;
void main() {
    gl_Position = vec4(position, 1.0);
}
"#;

const FRAG_SRC: &str = r#"

#version 450
#define GLSLIFY 1
#define LOWQUALITY 1

// UBO matches your Rust side
layout(set = 0, binding = 0, std140) uniform Params {
    vec4 iResolution;  // (width, height, pixelAspect, _)
    vec4 iTimeData;    // (iTime, iTimeDelta, iFrame_as_float, _)
    vec4 iMouse;       // (x, y, clickX, clickY)
};

#define iTime       (iTimeData.x)
#define iTimeDelta  (iTimeData.y)
#define iFrame      int(iTimeData.z)

layout(location = 0) out vec4 out_color;

// ===== all your helpers/utilities/noise/fbm/etc from the snippet =====
// Keep bodies exactly as you pasted (sdEllipsoid*, hash*, noise*, fbm*, clouds*, terrain*, trees*, renderSky, etc.)
// The only thing removed is the HISTORY/iChannel block at the end of mainImage.

#define ZERO (min(iFrame,0))

/* ... paste all your functions here unchanged, EXCEPT the reprojection/history section ... */
float sdEllipsoidY( in vec3 p, in vec2 r )
{
    float k0 = length(p/r.xyx);
    float k1 = length(p/(r.xyx*r.xyx));
    return k0*(k0-1.0)/k1;
}
float sdEllipsoid( in vec3 p, in vec3 r )
{
    float k0 = length(p/r);
    float k1 = length(p/(r*r));
    return k0*(k0-1.0)/k1;
}

// return smoothstep and its derivative
vec2 smoothstepd( float a, float b, float x)
{
	if( x<a ) return vec2( 0.0, 0.0 );
	if( x>b ) return vec2( 1.0, 0.0 );
    float ir = 1.0/(b-a);
    x = (x-a)*ir;
    return vec2( x*x*(3.0-2.0*x), 6.0*x*(1.0-x)*ir );
}

mat3 setCamera( in vec3 ro, in vec3 ta, float cr )
{
	vec3 cw = normalize(ta-ro);
	vec3 cp = vec3(sin(cr), cos(cr),0.0);
	vec3 cu = normalize( cross(cw,cp) );
	vec3 cv = normalize( cross(cu,cw) );
    return mat3( cu, cv, cw );
}

//==========================================================================================
// hashes (low quality, do NOT use in production)
//==========================================================================================

float hash1( vec2 p )
{
    p  = 50.0*fract( p*0.3183099 );
    return fract( p.x*p.y*(p.x+p.y) );
}

float hash1( float n )
{
    return fract( n*17.0*fract( n*0.3183099 ) );
}

vec2 hash2( vec2 p ) 
{
    const vec2 k = vec2( 0.3183099, 0.3678794 );
    float n = 111.0*p.x + 113.0*p.y;
    return fract(n*fract(k*n));
}

//==========================================================================================
// noises
//==========================================================================================

// value noise, and its analytical derivatives
vec4 noised( in vec3 x )
{
    vec3 p = floor(x);
    vec3 w = fract(x);
    #if 1
    vec3 u = w*w*w*(w*(w*6.0-15.0)+10.0);
    vec3 du = 30.0*w*w*(w*(w-2.0)+1.0);
    #else
    vec3 u = w*w*(3.0-2.0*w);
    vec3 du = 6.0*w*(1.0-w);
    #endif

    float n = p.x + 317.0*p.y + 157.0*p.z;
    
    float a = hash1(n+0.0);
    float b = hash1(n+1.0);
    float c = hash1(n+317.0);
    float d = hash1(n+318.0);
    float e = hash1(n+157.0);
	float f = hash1(n+158.0);
    float g = hash1(n+474.0);
    float h = hash1(n+475.0);

    float k0 =   a;
    float k1 =   b - a;
    float k2 =   c - a;
    float k3 =   e - a;
    float k4 =   a - b - c + d;
    float k5 =   a - c - e + g;
    float k6 =   a - b - e + f;
    float k7 = - a + b + c - d + e - f - g + h;

    return vec4( -1.0+2.0*(k0 + k1*u.x + k2*u.y + k3*u.z + k4*u.x*u.y + k5*u.y*u.z + k6*u.z*u.x + k7*u.x*u.y*u.z), 
                      2.0* du * vec3( k1 + k4*u.y + k6*u.z + k7*u.y*u.z,
                                      k2 + k5*u.z + k4*u.x + k7*u.z*u.x,
                                      k3 + k6*u.x + k5*u.y + k7*u.x*u.y ) );
}

float noise( in vec3 x )
{
    vec3 p = floor(x);
    vec3 w = fract(x);
    
    #if 1
    vec3 u = w*w*w*(w*(w*6.0-15.0)+10.0);
    #else
    vec3 u = w*w*(3.0-2.0*w);
    #endif
    


    float n = p.x + 317.0*p.y + 157.0*p.z;
    
    float a = hash1(n+0.0);
    float b = hash1(n+1.0);
    float c = hash1(n+317.0);
    float d = hash1(n+318.0);
    float e = hash1(n+157.0);
	float f = hash1(n+158.0);
    float g = hash1(n+474.0);
    float h = hash1(n+475.0);

    float k0 =   a;
    float k1 =   b - a;
    float k2 =   c - a;
    float k3 =   e - a;
    float k4 =   a - b - c + d;
    float k5 =   a - c - e + g;
    float k6 =   a - b - e + f;
    float k7 = - a + b + c - d + e - f - g + h;

    return -1.0+2.0*(k0 + k1*u.x + k2*u.y + k3*u.z + k4*u.x*u.y + k5*u.y*u.z + k6*u.z*u.x + k7*u.x*u.y*u.z);
}

vec3 noised( in vec2 x )
{
    vec2 p = floor(x);
    vec2 w = fract(x);
    #if 1
    vec2 u = w*w*w*(w*(w*6.0-15.0)+10.0);
    vec2 du = 30.0*w*w*(w*(w-2.0)+1.0);
    #else
    vec2 u = w*w*(3.0-2.0*w);
    vec2 du = 6.0*w*(1.0-w);
    #endif
    
    float a = hash1(p+vec2(0,0));
    float b = hash1(p+vec2(1,0));
    float c = hash1(p+vec2(0,1));
    float d = hash1(p+vec2(1,1));

    float k0 = a;
    float k1 = b - a;
    float k2 = c - a;
    float k4 = a - b - c + d;

    return vec3( -1.0+2.0*(k0 + k1*u.x + k2*u.y + k4*u.x*u.y), 
                 2.0*du * vec2( k1 + k4*u.y,
                            k2 + k4*u.x ) );
}

float noise( in vec2 x )
{
    vec2 p = floor(x);
    vec2 w = fract(x);
    #if 1
    vec2 u = w*w*w*(w*(w*6.0-15.0)+10.0);
    #else
    vec2 u = w*w*(3.0-2.0*w);
    #endif

    float a = hash1(p+vec2(0,0));
    float b = hash1(p+vec2(1,0));
    float c = hash1(p+vec2(0,1));
    float d = hash1(p+vec2(1,1));
    
    return -1.0+2.0*(a + (b-a)*u.x + (c-a)*u.y + (a - b - c + d)*u.x*u.y);
}

//==========================================================================================
// fbm constructions
//==========================================================================================

const mat3 m3  = mat3( 0.00,  0.80,  0.60,
                      -0.80,  0.36, -0.48,
                      -0.60, -0.48,  0.64 );
const mat3 m3i = mat3( 0.00, -0.80, -0.60,
                       0.80,  0.36, -0.48,
                       0.60, -0.48,  0.64 );
const mat2 m2 = mat2(  0.80,  0.60,
                      -0.60,  0.80 );
const mat2 m2i = mat2( 0.80, -0.60,
                       0.60,  0.80 );

//------------------------------------------------------------------------------------------

float fbm_4( in vec2 x )
{
    float f = 1.9;
    float s = 0.55;
    float a = 0.0;
    float b = 0.5;
    for( int i=ZERO; i<4; i++ )
    {
        float n = noise(x);
        a += b*n;
        b *= s;
        x = f*m2*x;
    }
	return a;
}

float fbm_4( in vec3 x )
{
    float f = 2.0;
    float s = 0.5;
    float a = 0.0;
    float b = 0.5;
    for( int i=ZERO; i<4; i++ )
    {
        float n = noise(x);
        a += b*n;
        b *= s;
        x = f*m3*x;
    }
	return a;
}

vec4 fbmd_7( in vec3 x )
{
    float f = 1.92;
    float s = 0.5;
    float a = 0.0;
    float b = 0.5;
    vec3  d = vec3(0.0);
    mat3  m = mat3(1.0,0.0,0.0,
                   0.0,1.0,0.0,
                   0.0,0.0,1.0);
    for( int i=ZERO; i<7; i++ )
    {
        vec4 n = noised(x);
        a += b*n.x;          // accumulate values		
        d += b*m*n.yzw;      // accumulate derivatives
        b *= s;
        x = f*m3*x;
        m = f*m3i*m;
    }
	return vec4( a, d );
}

vec4 fbmd_8( in vec3 x )
{
    float f = 2.0;
    float s = 0.65;
    float a = 0.0;
    float b = 0.5;
    vec3  d = vec3(0.0);
    mat3  m = mat3(1.0,0.0,0.0,
                   0.0,1.0,0.0,
                   0.0,0.0,1.0);
    for( int i=ZERO; i<8; i++ )
    {
        vec4 n = noised(x);
        a += b*n.x;          // accumulate values		
        if( i<4 )
        d += b*m*n.yzw;      // accumulate derivatives
        b *= s;
        x = f*m3*x;
        m = f*m3i*m;
    }
	return vec4( a, d );
}

float fbm_9( in vec2 x )
{
    float f = 1.9;
    float s = 0.55;
    float a = 0.0;
    float b = 0.5;
    for( int i=ZERO; i<9; i++ )
    {
        float n = noise(x);
        a += b*n;
        b *= s;
        x = f*m2*x;
    }
    
	return a;
}

vec3 fbmd_9( in vec2 x )
{
    float f = 1.9;
    float s = 0.55;
    float a = 0.0;
    float b = 0.5;
    vec2  d = vec2(0.0);
    mat2  m = mat2(1.0,0.0,0.0,1.0);
    for( int i=ZERO; i<9; i++ )
    {
        vec3 n = noised(x);
        a += b*n.x;          // accumulate values		
        d += b*m*n.yz;       // accumulate derivatives
        b *= s;
        x = f*m2*x;
        m = f*m2i*m;
    }

	return vec3( a, d );
}

//==========================================================================================
// specifics to the actual painting
//==========================================================================================


//------------------------------------------------------------------------------------------
// global
//------------------------------------------------------------------------------------------

const vec3  kSunDir = vec3(-0.624695,0.468521,-0.624695);
const float kMaxTreeHeight = 4.8;
const float kMaxHeight = 840.0;

vec3 fog( in vec3 col, float t )
{
    vec3 ext = exp2(-t*0.00025*vec3(1,1.5,4)); 
    return col*ext + (1.0-ext)*vec3(0.55,0.55,0.58); // 0.55
}

//------------------------------------------------------------------------------------------
// clouds
//------------------------------------------------------------------------------------------

vec4 cloudsFbm( in vec3 pos )
{
    return fbmd_8(pos*0.0015+vec3(2.0,1.1,1.0)+0.07*vec3(iTime,0.5*iTime,-0.15*iTime));
}

vec4 cloudsMap( in vec3 pos, out float nnd )
{
    float d = abs(pos.y-900.0)-40.0;
    vec3 gra = vec3(0.0,sign(pos.y-900.0),0.0);
    
    vec4 n = cloudsFbm(pos);
    d += 400.0*n.x * (0.7+0.3*gra.y);
    
    if( d>0.0 ) return vec4(-d,0.0,0.0,0.0);
    
    nnd = -d;
    d = min(-d/100.0,0.25);
    
    //gra += 0.1*n.yzw *  (0.7+0.3*gra.y);
    
    return vec4( d, gra );
}

float cloudsShadowFlat( in vec3 ro, in vec3 rd )
{
    float t = (900.0-ro.y)/rd.y;
    if( t<0.0 ) return 1.0;
    vec3 pos = ro + rd*t;
    return cloudsFbm(pos).x;
}

//------------------------------------------------------------------------------------------
// terrain
//------------------------------------------------------------------------------------------

vec2 terrainMap( in vec2 p )
{
    float e = fbm_9( p/2000.0 + vec2(1.0,-2.0) );
    float a = 1.0-smoothstep( 0.12, 0.13, abs(e+0.12) ); // flag high-slope areas (-0.25, 0.0)
    e = 600.0*e + 600.0;
    
    // cliff
    e += 90.0*smoothstep( 552.0, 594.0, e );
    //e += 90.0*smoothstep( 550.0, 600.0, e );
    
    return vec2(e,a);
}

vec4 terrainMapD( in vec2 p )
{
    vec3 e = fbmd_9( p/2000.0 + vec2(1.0,-2.0) );
    e.x  = 600.0*e.x + 600.0;
    e.yz = 600.0*e.yz;

    // cliff
    vec2 c = smoothstepd( 550.0, 600.0, e.x );
	e.x  = e.x  + 90.0*c.x;
	e.yz = e.yz + 90.0*c.y*e.yz;     // chain rule
    
    e.yz /= 2000.0;
    return vec4( e.x, normalize( vec3(-e.y,1.0,-e.z) ) );
}

vec3 terrainNormal( in vec2 pos )
{
#if 1
    return terrainMapD(pos).yzw;
#else    
    vec2 e = vec2(0.03,0.0);
	return normalize( vec3(terrainMap(pos-e.xy).x - terrainMap(pos+e.xy).x,
                           2.0*e.x,
                           terrainMap(pos-e.yx).x - terrainMap(pos+e.yx).x ) );
#endif    
}

float terrainShadow( in vec3 ro, in vec3 rd, in float mint )
{
    float res = 1.0;
    float t = mint;
#ifdef LOWQUALITY
    for( int i=ZERO; i<32; i++ )
    {
        vec3  pos = ro + t*rd;
        vec2  env = terrainMap( pos.xz );
        float hei = pos.y - env.x;
        res = min( res, 32.0*hei/t );
        if( res<0.0001 || pos.y>kMaxHeight ) break;
        t += clamp( hei, 2.0+t*0.1, 100.0 );
    }
#else
    for( int i=ZERO; i<128; i++ )
    {
        vec3  pos = ro + t*rd;
        vec2  env = terrainMap( pos.xz );
        float hei = pos.y - env.x;
        res = min( res, 32.0*hei/t );
        if( res<0.0001 || pos.y>kMaxHeight  ) break;
        t += clamp( hei, 0.5+t*0.05, 25.0 );
    }
#endif
    return clamp( res, 0.0, 1.0 );
}


vec4 renderClouds( in vec3 ro, in vec3 rd, float tmin, float tmax, inout float resT, in vec2 px )
{
    vec4 sum = vec4(0.0);

    // bounding volume!!
    float tl = ( 600.0-ro.y)/rd.y;
    float th = (1200.0-ro.y)/rd.y;
    if( tl>0.0 ) tmin = max( tmin, tl ); else return sum;
    if( th>0.0 ) tmax = min( tmax, th );

    float t = tmin;
    //t += 1.0*hash1(gl_FragCoord.xy);
    float lastT = -1.0;
    float thickness = 0.0;
    for(int i=ZERO; i<128; i++)
    { 
        vec3  pos = ro + t*rd; 
        float nnd;
        vec4  denGra = cloudsMap( pos, nnd ); 
        float den = denGra.x;
        float dt = max(0.2,0.011*t);
        //dt *= hash1(px+float(i));
        if( den>0.001 ) 
        { 
            float kk;
            cloudsMap( pos+kSunDir*70.0, kk );
            float sha = 1.0-smoothstep(-200.0,200.0,kk); sha *= 1.5;
            
            vec3 nor = normalize(denGra.yzw);
            float dif = clamp( 0.4+0.6*dot(nor,kSunDir), 0.0, 1.0 )*sha; 
            float fre = clamp( 1.0+dot(nor,rd), 0.0, 1.0 )*sha;
            float occ = 0.2+0.7*max(1.0-kk/200.0,0.0) + 0.1*(1.0-den);
            // lighting
            vec3 lin  = vec3(0.0);
                 lin += vec3(0.70,0.80,1.00)*1.0*(0.5+0.5*nor.y)*occ;
                 lin += vec3(0.10,0.40,0.20)*1.0*(0.5-0.5*nor.y)*occ;
                 lin += vec3(1.00,0.95,0.85)*3.0*dif*occ + 0.1;

            // color
            vec3 col = vec3(0.8,0.8,0.8)*0.45;

            col *= lin;

            col = fog( col, t );

            // front to back blending    
            float alp = clamp(den*0.5*0.125*dt,0.0,1.0);
            col.rgb *= alp;
            sum = sum + vec4(col,alp)*(1.0-sum.a);

            thickness += dt*den;
            if( lastT<0.0 ) lastT = t;            
        }
        else 
        {
            dt = abs(den)+0.2;

        }
        t += dt;
        if( sum.a>0.995 || t>tmax ) break;
    }
    
    //resT = min(resT, (150.0-ro.y)/rd.y );
    if( lastT>0.0 ) resT = min(resT,lastT);
    //if( lastT>0.0 ) resT = mix( resT, lastT, sum.w );
    
    
    sum.xyz += max(0.0,1.0-0.0125*thickness)*vec3(1.00,0.60,0.40)*0.3*pow(clamp(dot(kSunDir,rd),0.0,1.0),32.0);

    return clamp( sum, 0.0, 1.0 );
}





vec2 raymarchTerrain( in vec3 ro, in vec3 rd, float tmin, float tmax )
{
    // bounding plane
    float tp = (kMaxHeight+kMaxTreeHeight-ro.y)/rd.y;
    if( tp>0.0 ) tmax = min( tmax, tp );
    
    // raymarch
    float dis, th;
    float t2 = -1.0;
    float t = tmin; 
    float ot = t;
    float odis = 0.0;
    float odis2 = 0.0;
    for( int i=ZERO; i<400; i++ )
    {
        th = 0.001*t;

        vec3  pos = ro + t*rd;
        vec2  env = terrainMap( pos.xz );
        float hei = env.x;

        // tree envelope
        float dis2 = pos.y - (hei+kMaxTreeHeight*1.1);
        if( dis2<th ) 
        {
            if( t2<0.0 )
            {
                t2 = ot + (th-odis2)*(t-ot)/(dis2-odis2); // linear interpolation for better accuracy
            }
        }
        odis2 = dis2;
        
        // terrain
        dis = pos.y - hei;
        if( dis<th ) break;
        
        ot = t;
        odis = dis;
        t += dis*0.8*(1.0-0.75*env.y); // slow down in step areas
        if( t>tmax ) break;
    }

    if( t>tmax ) t = -1.0;
    else t = ot + (th-odis)*(t-ot)/(dis-odis); // linear interpolation for better accuracy
    
    return vec2(t,t2);
}


//------------------------------------------------------------------------------------------
// trees (calmer, distance-aware)
//------------------------------------------------------------------------------------------

float treesMap( in vec3 p, in float rt, out float oHei, out float oMat, out float oDis )
{
    oHei = 1.0;
    oDis = 0.0;
    oMat = 0.0;

    float base = terrainMap(p.xz).x;

    float bb = fbm_4(p.xz*0.075);

    float d = 20.0;
    vec2 n = floor( p.xz/2.0 );
    vec2 f = fract( p.xz/2.0 );
    for( int j=0; j<=1; j++ )
    for( int i=0; i<=1; i++ )
    {
        vec2  g = vec2( float(i), float(j) ) - step(f,vec2(0.5));
        vec2  o = hash2( n + g );
        vec2  v = hash2( n + g + vec2(13.1,71.7) );
        vec2  r = g - f + o;

        float height = kMaxTreeHeight * (0.4+0.8*v.x);
        float width  = 0.5 + 0.2*v.x + 0.3*v.y;

        if( bb<0.0 ) width *= 0.5; else height *= 0.7;

        // widen crowns a bit at distance for better sub-pixel stability
        float lodWide = smoothstep(150.0, 600.0, rt);
        width *= mix(1.0, 1.25, lodWide);

        vec3  q = vec3(r.x, p.y - base - height*0.5, r.y);

        float k = sdEllipsoidY( q, vec2(width, 0.5*height) );

        if( k<d )
        {
            d = k;
            oMat = 0.5*hash1(n+g+111.0);
            if( bb>0.0 ) oMat += 0.5;
            oHei = (p.y - base)/height;
            oHei *= 0.5 + 0.5*length(q) / width;
        }
    }

    // distance-aware crown distortion (softer & lower-freq far away)
    if( rt < 1200.0 )
    {
        p.y -= 600.0;

        // reduce frequency and strength with distance
        float lodAtt = smoothstep(80.0, 400.0, rt);
        float fr     = mix(3.0, 1.6, lodAtt);       // <= lower freq far
        float s      = fbm_4( p * fr ); s = s*s;

        float att    = 1.0 - smoothstep(100.0, 1200.0, rt);
        d    += mix(4.0, 1.5, lodAtt) * s * att;   // <= weaker far
        oDis  = s * att * (1.0 - 0.7*lodAtt);      // <= fade displacement tag too
    }

    return d;
}

float treesShadow( in vec3 ro, in vec3 rd )
{
    float res = 1.0;
    float t = 0.02;
#ifdef LOWQUALITY
    for( int i=ZERO; i<64; i++ )
    {
        float kk1, kk2, kk3;
        vec3 pos = ro + rd*t;
        float h  = treesMap( pos, t, kk1, kk2, kk3 );

        // impose a minimum step based on world-space pixel size to avoid thrashing
        float minStep = 0.4 * (t / iResolution.y);
        t += max(h, minStep);

        res = min( res, 32.0*h/max(t, 1e-3) );
        if( res<0.001 || t>50.0 || pos.y>kMaxHeight+kMaxTreeHeight ) break;
    }
#else
    for( int i=ZERO; i<150; i++ )
    {
        float kk1, kk2, kk3;
        float h  = treesMap( ro + rd*t, t, kk1, kk2, kk3 );
        t += h;
        res = min( res, 32.0*h/max(t, 1e-3) );
        if( res<0.001 || t>120.0 ) break;
    }
#endif
    return clamp( res, 0.0, 1.0 );
}

vec3 treesNormal( in vec3 pos, in float t )
{
    float kk1, kk2, kk3;

    // pixel-sized epsilon: grow with distance so normals don’t flicker on sub-pixel crowns
    float eps = max(0.005, 2.0 * (t / iResolution.y));

    // tetrahedral gradient (keeps compiler from inlining 4x)
    vec3 n = vec3(0.0);
    for( int i=ZERO; i<4; i++ )
    {
        vec3 e = 0.5773 * (2.0 * vec3((((i+3)>>1)&1), ((i>>1)&1), (i&1)) - 1.0);
        n += e * treesMap(pos + eps * e, t, kk1, kk2, kk3);
    }
    return normalize(n);
}

//------------------------------------------------------------------------------------------
// sky
//------------------------------------------------------------------------------------------

vec3 renderSky( in vec3 ro, in vec3 rd )
{
    // background sky     
    //vec3 col = vec3(0.45,0.6,0.85)/0.85 - rd.y*vec3(0.4,0.36,0.4);
    //vec3 col = vec3(0.4,0.6,1.1) - rd.y*0.4;
    vec3 col = vec3(0.42,0.62,1.1) - rd.y*0.4;

    // clouds
    float t = (2500.0-ro.y)/rd.y;
    if( t>0.0 )
    {
        vec2 uv = (ro+t*rd).xz;
        float cl = fbm_9( uv*0.00104 );
        float dl = smoothstep(-0.2,0.6,cl);
        col = mix( col, vec3(1.0), 0.12*dl );
    }
    
	// sun glare    
    float sun = clamp( dot(kSunDir,rd), 0.0, 1.0 );
    col += 0.2*vec3(1.0,0.6,0.3)*pow( sun, 32.0 );
    
	return col;
}
// --- mainImage without HISTORY/iChannel ---
void mainImage( out vec4 fragColor, in vec2 fragCoord )
{
    // vec2 o = /* jitter */ (vec2(fract(sin(dot(vec2(iFrame,1),vec2(12.9898,78.233))) * 43758.5453)) - 0.5);
    // vec2 p = (2.0*(fragCoord+o)-iResolution.xy) / iResolution.y;

// replace the jittered o with zero
vec2 o = vec2(0.0);
// and don't add it to fragCoord
vec2 p = (2.0*fragCoord - iResolution.xy) / iResolution.y;
    // camera
    float time = iTime;
    vec3 ro = vec3(0.0, 401.5, 6.0);
    vec3 ta = vec3(0.0, 403.5, -90.0 + ro.z );
    ro.x -= 80.0*sin(0.01*time);
    ta.x -= 86.0*sin(0.01*time);

    mat3 ca = setCamera( ro, ta, 0.0 );
    vec3 rd = ca * normalize( vec3(p,1.5));

    float resT = 2000.0;

    // sky
    vec3 col = renderSky( ro, rd );

    // terrain + trees (unchanged logic)
    {
        const float tmax = 2000.0;
        int   obj = 0;
        vec2 t = raymarchTerrain( ro, rd, 15.0, tmax );
        if( t.x>0.0 ) { resT = t.x; obj = 1; }

        float hei, mid, displa;
        if( t.y>0.0 )
        {
            float tf = t.y;
            float tfMax = (t.x>0.0)?t.x:tmax;
            for(int i=ZERO; i<64; i++)
            {
                vec3  pos = ro + tf*rd;
                float dis = treesMap( pos, tf, hei, mid, displa );
                float pixelWorld = tf / iResolution.y;
if( dis < max(0.000125*tf, 0.6*pixelWorld) ) break;
                tf += dis;
                if( tf>tfMax ) break;
            }
            if( tf<tfMax ) { resT = tf; obj = 2; }
        }

        if( obj>0 )
        {
            vec3 pos  = ro + resT*rd;
            vec3 epos = pos + vec3(0.0,4.8,0.0);

            float sha1  = terrainShadow( pos+vec3(0,0.02,0), kSunDir, 0.02 );
            sha1 *= smoothstep(-0.325,-0.075,cloudsShadowFlat(epos, kSunDir));

            #ifndef LOWQUALITY
            float sha2  = treesShadow( pos+vec3(0,0.02,0), kSunDir );
            #endif

            vec3 tnor = terrainNormal( pos.xz );
            vec3 nor;
            vec3 speC = vec3(1.0);

            if( obj==1 )
            {
                nor = normalize( tnor + 0.8*(1.0-abs(tnor.y))*0.8*fbmd_7( (pos-vec3(0,600,0))*0.15*vec3(1.0,0.2,1.0) ).yzw );
                vec3 base = vec3(0.18,0.12,0.10)*.85;
                base = 1.0*mix( base, vec3(0.1,0.1,0.0)*0.2, smoothstep(0.7,0.9,nor.y) );
                float dif = clamp( dot( nor, kSunDir), 0.0, 1.0 ) * sha1;
                #ifndef LOWQUALITY
                dif *= sha2;
                #endif
                float bac = clamp( dot(normalize(vec3(-kSunDir.x,0.0,-kSunDir.z)),nor), 0.0, 1.0 );
                float foc = clamp( (pos.y/2.0-180.0)/130.0, 0.0,1.0 );
                float dom = clamp( 0.5 + 0.5*nor.y, 0.0, 1.0 );
                vec3  lin  = 1.0*0.2*mix(0.1*vec3(0.1,0.2,0.1),vec3(0.7,0.9,1.5)*3.0,dom)*foc
                           + 1.0*8.5*vec3(1.0,0.9,0.8)*dif
                           + 1.0*0.27*vec3(1.1,1.0,0.9)*bac*foc;
                speC = vec3(4.0)*dif*smoothstep(20.0,0.0,abs(pos.y/2.0-310.0)-20.0);
                col = base * lin;
            }
            else // trees
            {
                vec3 gnor = treesNormal( pos, resT );
                nor = normalize( gnor + 2.0*tnor );

                float hei, mid, displa; // if needed by your material block
                vec3  ref = reflect(rd,nor);
                float occ = 1.0; // simplified
                float dif = clamp( 0.1 + 0.9*dot( nor, kSunDir), 0.0, 1.0 ) * sha1;
                #ifdef LOWQUALITY
                float sha2  = treesShadow( pos+kSunDir*0.1, kSunDir );
                dif *= (0.7 + 0.3*sha2);
                #endif
                float dom = clamp( 0.5 + 0.5*nor.y, 0.0, 1.0 );
                float bac = clamp( 0.5+0.5*dot(normalize(vec3(-kSunDir.x,0.0,-kSunDir.z)),nor), 0.0, 1.0 );
                float fre = clamp(1.0+dot(nor,rd),0.0,1.0);

                vec3 lin  = 12.0*vec3(1.2,1.0,0.7)*dif*occ*(2.5-1.5*smoothstep(0.0,120.0,resT))
                          + 0.55*mix(0.1*vec3(0.1,0.2,0.0),vec3(0.6,1.0,1.0),dom*occ)
                          + 0.07*vec3(1.0,1.0,0.9)*bac*occ
                          + 1.10*vec3(0.9,1.0,0.8)*pow(fre,5.0)*occ*(1.0-smoothstep(100.0,200.0,resT));
                vec3 speC2 = dif*vec3(1.0,1.1,1.5)*1.2;

                // very simplified tree albedo from your block
                vec3 base = vec3(0.25,0.20,0.06);
                col = base * lin;
                speC = speC2;
            }

            vec3 ref = reflect(rd,nor);
            float fre = clamp(1.0+dot(nor,rd),0.0,1.0);
            float spe = 3.0*pow( clamp(dot(ref,kSunDir),0.0, 1.0), 9.0 )*(0.05+0.95*pow(fre,5.0));
            col += spe*speC;
            col = fog(col,resT);
        }
    }

    // clouds
    {
        vec4 res = renderClouds( ro, rd, 0.0, resT, resT, fragCoord );
        col = col*(1.0-res.w) + res.xyz;
    }

    
// glare (ok)
float sun = clamp(dot(kSunDir,rd), 0.0, 1.0);
col += 0.25*vec3(0.8,0.4,0.2)*pow(sun, 4.0);

// tone & contrast (stay in linear)
col = clamp(col*1.1 - 0.02, 0.0, 1.0);
col = col*col*(3.0-2.0*col);

// mild color grade is fine in linear
col = pow(col, vec3(1.0,0.92,1.0));
col *= vec3(1.02,0.99,0.9);
col.z += 0.1;

// DO NOT gamma encode here when your surface is sRGB
fragColor = vec4(col, 1.0);

}

void main() {
    mainImage(out_color, gl_FragCoord.xy);
}
"#;
