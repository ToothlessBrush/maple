use std::time::Instant;

use bytemuck::{Pod, Zeroable};
use maple_app::{app::App, plugin::Plugin};
use maple_renderer::core::{
    buffer::Buffer,
    descriptor_set::{
        DescriptorBindingType, DescriptorSet, DescriptorSetDescriptor, DescriptorSetLayout,
        DescriptorSetLayoutDescriptor, StageFlags,
    },
    pipeline::RenderPipeline,
    render_pass::{RenderPass, RenderPassDescriptor},
    renderer::Renderer,
    shader::{GraphicsShader, ShaderPair},
};
use maple_renderer::types::Vertex;

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
    App::new().add_plugin(ShaderToy).run();
}

struct ShaderToy;

impl Plugin for ShaderToy {
    fn init(&self, app: &mut App<maple_app::app::Running>) {
        app.add_renderpass(MainPass::new());
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Pass
// ─────────────────────────────────────────────────────────────────────────────

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

impl RenderPass for MainPass {
    fn setup(&mut self, renderer: &Renderer) -> RenderPassDescriptor {
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

        let vbuf = renderer.create_vertex_buffer(&vertices);
        let ibuf = renderer.create_index_buffer(&indices);
        let pbuf = renderer.create_uniform_buffer(&self.params);

        // Descriptor set 0: UBO
        let layout = renderer.create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
            label: Some("shadertoy_params_layout"),
            visibility: StageFlags::FRAGMENT,
            layout: &[DescriptorBindingType::UniformBuffer],
        });

        let set = renderer.build_descriptor_set(
            DescriptorSet::builder(&layout)
                .label("shadertoy_params_set")
                .uniform(0, &pbuf),
        );

        // Shaders
        let shader = renderer.create_shader_pair(ShaderPair::Glsl {
            vert: VERT_SRC,
            frag: FRAG_SRC,
        });

        // Store resources
        self.vertex_buffer = Some(vbuf);
        self.index_buffer = Some(ibuf);
        self.params_buffer = Some(pbuf);
        self.params_layout = Some(layout.clone());
        self.params_set = Some(set);

        RenderPassDescriptor {
            name: "shadertoy pass",
            shader,
            descriptor_set_layouts: vec![layout],
        }
    }

    fn draw(
        &mut self,
        renderer: &Renderer,
        pipeline: &RenderPipeline,
        _drawables: &[&dyn maple_renderer::types::drawable::Drawable],
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
        renderer.write_buffer(self.params_buffer.as_ref().unwrap(), &self.params)?;

        // Draw
        renderer.render(pipeline, |mut fb| {
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

// UBO matches Rust ShadertoyParams (vec4-only, std140-safe)
layout(set = 0, binding = 0, std140) uniform Params {
    vec4 iResolution;  // (width, height, pixelAspect, _)
    vec4 iTimeData;    // (iTime, iTimeDelta, iFrame_as_float, _)
    vec4 iMouse;       // (x, y, clickX, clickY)
};

#define iTime       (iTimeData.x)
#define iTimeDelta  (iTimeData.y)
#define iFrame      int(iTimeData.z)

layout(location = 0) out vec4 out_color;

// ─────────────────────── lowest-level utils (no external deps) ───────────────────────
vec2 rotate2D(vec2 p, float a) { return p * mat2(cos(a), -sin(a), sin(a), cos(a)); }

float random_2281831123(vec2 co) {
  float a = 12.9898;
  float b = 78.233;
  float c = 43758.5453;
  float dt= dot(co.xy ,vec2(a,b));
  float sn= mod(dt,3.14);
  return fract(sin(sn) * c);
}

float sdBox_1117569599(vec3 position, vec3 dimensions) {
  vec3 d = abs(position) - dimensions;
  return min(max(d.x, max(d.y,d.z)), 0.0) + length(max(d, 0.0));
}

float fogFactorExp2_529295689(const float dist, const float density) {
  const float LOG2 = -1.442695;
  float d = density * dist;
  return 1.0 - clamp(exp2(d * d * LOG2), 0.0, 1.0);
}

float intersectPlane(vec3 ro, vec3 rd, vec3 nor, float dist) {
  float denom = dot(rd, nor);
  return -(dot(ro, nor) + dist) / denom;
}

// icosahedral support (constants first, then function)
vec3 n4  = vec3( 0.577,  0.577,  0.577);
vec3 n5  = vec3(-0.577,  0.577,  0.577);
vec3 n6  = vec3( 0.577, -0.577,  0.577);
vec3 n7  = vec3( 0.577,  0.577, -0.577);
vec3 n8  = vec3( 0.000,  0.357,  0.934);
vec3 n9  = vec3( 0.000, -0.357,  0.934);
vec3 n10 = vec3( 0.934,  0.000,  0.357);
vec3 n11 = vec3(-0.934,  0.000,  0.357);
vec3 n12 = vec3( 0.357,  0.934,  0.000);
vec3 n13 = vec3(-0.357,  0.934,  0.000);

float icosahedral(vec3 p, float r) {
  float s = abs(dot(p,n4));
  s = max(s, abs(dot(p,n5)));
  s = max(s, abs(dot(p,n6)));
  s = max(s, abs(dot(p,n7)));
  s = max(s, abs(dot(p,n8)));
  s = max(s, abs(dot(p,n9)));
  s = max(s, abs(dot(p,n10)));
  s = max(s, abs(dot(p,n11)));
  s = max(s, abs(dot(p,n12)));
  s = max(s, abs(dot(p,n13)));
  return s - r;
}

// ─────────────────────── maps FIRST (so later code can call them) ───────────────────────
vec2 mapRefract(vec3 p) {
  float d  = icosahedral(p, 1.0);
  float id = 0.0;
  return vec2(d, id);
}

vec2 mapSolid(vec3 p) {
  p.xz = rotate2D(p.xz, iTime * 1.25);
  p.yx = rotate2D(p.yx, iTime * 1.85);
  p.y += sin(iTime) * 0.25;
  p.x += cos(iTime) * 0.25;

  float d  = length(p) - 0.25;
  float id = 1.0;
  float pulse = pow(sin(iTime * 2.) * 0.5 + 0.5, 9.0) * 2.0;
  d = mix(d, sdBox_1117569599(p, vec3(0.175)), pulse);
  return vec2(d, id);
}

// ─────────────────────── mid-level helpers (now safe to call maps) ───────────────────────
vec3 calcNormalRefract(vec3 pos, float eps) {
  const vec3 v1 = vec3( 1.0,-1.0,-1.0);
  const vec3 v2 = vec3(-1.0,-1.0, 1.0);
  const vec3 v3 = vec3(-1.0, 1.0,-1.0);
  const vec3 v4 = vec3( 1.0, 1.0, 1.0);
  return normalize(
      v1 * mapRefract(pos + v1*eps).x +
      v2 * mapRefract(pos + v2*eps).x +
      v3 * mapRefract(pos + v3*eps).x +
      v4 * mapRefract(pos + v4*eps).x
  );
}

vec3 calcNormalSolid(vec3 pos, float eps) {
  const vec3 v1 = vec3( 1.0,-1.0,-1.0);
  const vec3 v2 = vec3(-1.0,-1.0, 1.0);
  const vec3 v3 = vec3(-1.0, 1.0,-1.0);
  const vec3 v4 = vec3( 1.0, 1.0, 1.0);
  return normalize(
      v1 * mapSolid(pos + v1*eps).x +
      v2 * mapSolid(pos + v2*eps).x +
      v3 * mapSolid(pos + v3*eps).x +
      v4 * mapSolid(pos + v4*eps).x
  );
}

vec3 calcNormalRefract(vec3 pos) { return calcNormalRefract(pos, 0.002); }
vec3 calcNormalSolid  (vec3 pos) { return calcNormalSolid  (pos, 0.002); }

vec2 marchRefract(vec3 ro, vec3 rd, float maxd, float precis) {
  float latest = precis * 2.0;
  float dist   = 0.0;
  float type   = -1.0;
  for (int i = 0; i < 50; ++i) {
    if (latest < precis || dist > maxd) break;
    vec2 res = mapRefract(ro + rd * dist);
    latest = res.x; type = res.y; dist += latest;
  }
  return (dist < maxd) ? vec2(dist, type) : vec2(-1.0);
}

vec2 marchSolid(vec3 ro, vec3 rd, float maxd, float precis) {
  float latest = precis * 2.0;
  float dist   = 0.0;
  float type   = -1.0;
  for (int i = 0; i < 60; ++i) {
    if (latest < precis || dist > maxd) break;
    vec2 res = mapSolid(ro + rd * dist);
    latest = res.x; type = res.y; dist += latest;
  }
  return (dist < maxd) ? vec2(dist, type) : vec2(-1.0);
}

vec2 marchRefract(vec3 ro, vec3 rd) { return marchRefract(ro, rd, 20.0, 0.001); }
vec2 marchSolid  (vec3 ro, vec3 rd) { return marchSolid  (ro, rd, 20.0, 0.001); }

// camera / screen helpers (don’t call maps)
vec2 squareFrame(vec2 screenSize, vec2 coord) {
  vec2 p = 2.0 * (coord / screenSize) - 1.0;
  p.x *= screenSize.x / screenSize.y;
  return p;
}

mat3 calcLookAtMatrix_1535977339(vec3 origin, vec3 target, float roll) {
  vec3 rr = vec3(sin(roll), cos(roll), 0.0);
  vec3 ww = normalize(target - origin);
  vec3 uu = normalize(cross(ww, rr));
  vec3 vv = normalize(cross(uu, ww));
  return mat3(uu, vv, ww);
}

vec3 getRay_cam(mat3 camMat, vec2 screenPos, float lensLength) {
  return normalize(camMat * vec3(screenPos, lensLength));
}

vec3 getRay(vec3 origin, vec3 target, vec2 screenPos, float lensLength) {
  mat3 cam = calcLookAtMatrix_1535977339(origin, target, 0.0);
  return getRay_cam(cam, screenPos, lensLength);
}

void orbitCamera(
  in float camAngle,
  in float camHeight,
  in float camDistance,
  in vec2 screenResolution,
  out vec3 rayOrigin,
  out vec3 rayDirection,
  in vec2 coord
) {
  vec2 screenPos = squareFrame(screenResolution, coord);
  vec3 rayTarget = vec3(0.0);
  rayOrigin = vec3(camDistance * sin(camAngle), camHeight, camDistance * cos(camAngle));
  rayDirection = getRay(rayOrigin, rayTarget, screenPos, 2.0);
}

// lighting utils
float beckmannDistribution_2315452051(float x, float roughness) {
  float NdotH = max(x, 0.0001);
  float cos2A = NdotH * NdotH;
  float tan2A = (cos2A - 1.0) / cos2A;
  float r2 = roughness * roughness;
  float denom = 3.141592653589793 * r2 * cos2A * cos2A;
  return exp(tan2A / r2) / denom;
}

float cookTorranceSpecular_1460171947(
  vec3 l, vec3 v, vec3 n, float roughness, float fresnelPow) {

  float VdotN = max(dot(v, n), 0.0);
  float LdotN = max(dot(l, n), 0.0);
  vec3  H     = normalize(l + v);

  float NdotH = max(dot(n, H), 0.0);
  float VdotH = max(dot(v, H), 0.000001);
  float LdotH = max(dot(l, H), 0.000001);
  float G1 = (2.0 * NdotH * VdotN) / VdotH;
  float G2 = (2.0 * NdotH * LdotN) / LdotH;
  float G  = min(1.0, min(G1, G2));

  float D = beckmannDistribution_2315452051(NdotH, roughness);
  float F = pow(1.0 - VdotN, fresnelPow);

  return G * F * D / max(3.14159265 * VdotN, 0.000001);
}

// palette + bg
vec3 palette( in float t, in vec3 a, in vec3 b, in vec3 c, in vec3 d ) {
    return a + b*cos( 6.28318*(c*t+d) );
}

vec3 bg(vec3 ro, vec3 rd) {
  vec3 col = 0.1 + palette(
    clamp((random_2281831123(rd.xz + sin(iTime * 0.1)) * 0.5 + 0.5) * 0.035 - rd.y * 0.5 + 0.35, -1.0, 1.0),
    vec3(0.5, 0.45, 0.55),
    vec3(0.5, 0.5, 0.5),
    vec3(1.05, 1.0, 1.0),
    vec3(0.275, 0.2, 0.19)
  );

  float t = intersectPlane(ro, rd, vec3(0, 1, 0), 4.0);
  if (t > 0.0) {
    vec3 p = ro + rd * t;
    float g = (1.0 - pow(abs(sin(p.x) * cos(p.z)), 0.25));
    col += (1.0 - fogFactorExp2_529295689(t, 0.04)) * g * vec3(5, 4, 2) * 0.075;
  }
  return col;
}

// ─────────────────────────────────── mainImage ───────────────────────────────────
void mainImage(out vec4 fragColor, in vec2 fragCoord) {
  vec3 ro, rd;

  vec2  uv       = squareFrame(iResolution.xy, fragCoord);
  float dist     = 4.5;
  float rotation = (iMouse.z > 0.0) ? 6.0 * iMouse.x / iResolution.x : iTime * 0.45;
  float height   = (iMouse.z > 0.0) ? 5.0 * (iMouse.y / iResolution.y * 2.0 - 1.0) : -0.2;

  orbitCamera(rotation, height, dist, iResolution.xy, ro, rd, fragCoord);

  vec3 color = bg(ro, rd);
  vec2 t = marchRefract(ro, rd);
  if (t.x > -0.5) {
    vec3 pos  = ro + rd * t.x;
    vec3 nor  = calcNormalRefract(pos);

    vec3 ldir1 = normalize(vec3( 0.8,  1.0, 0.0));
    vec3 ldir2 = normalize(vec3(-0.4, -1.3, 0.0));
    vec3 lcol1 = vec3(0.6, 0.5, 1.1);
    vec3 lcol2 = vec3(1.4, 0.9, 0.8) * 0.7;

    vec3 ref = refract(rd, nor, 0.97);
    vec2 u = marchSolid(ro + ref * 0.1, ref);
    if (u.x > -0.5) {
      vec3 pos2 = ro + ref * u.x;
      vec3 nor2 = calcNormalSolid(pos2);
      float spec = cookTorranceSpecular_1460171947(ldir1, -ref, nor2, 0.6, 0.95) * 2.0;
      float diff1 = 0.05 + max(0.0, dot(ldir1, nor2));
      float diff2 = max(0.0, dot(ldir2, nor2));
      color = spec + (diff1 * lcol1 + diff2 * lcol2);
    } else {
      color = bg(ro + ref * 0.1, ref) * 1.1;
    }

    color += color * cookTorranceSpecular_1460171947(ldir1, -rd, nor, 0.2, 0.9) * 2.0;
    color += 0.05;
  }

  float vignette = 1.0 - max(0.0, dot(uv * 0.155, uv));
  color.r = smoothstep( 0.05, 0.995, color.r);
  color.b = smoothstep(-0.05, 0.95,  color.b);
  color.g = smoothstep(-0.1, 0.95,  color.g);
  color.b *= vignette;

  fragColor = vec4(color, clamp(t.x, 0.5, 1.0));
}


void main() {
  // flip Y so (0,0) is bottom-left like Shadertoy
  vec2 fc = vec2(gl_FragCoord.x, iResolution.y - gl_FragCoord.y);
  mainImage(out_color, fc);
}

"#;
