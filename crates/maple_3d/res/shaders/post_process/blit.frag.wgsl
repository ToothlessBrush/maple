 @group(0) @binding(0) var scene_texture: texture_2d<f32>;                                                                                                                                                                                                                       
  @group(0) @binding(1) var bloom_texture: texture_2d<f32>;                                                                                                                                                                                                                       
  @group(0) @binding(2) var tex_sampler: sampler;                                                                                                                                                                                                                                 
                                                                                                                                                                                                                                                                                  
struct Uniforms {
    bloom_intensity: f32,
    exposure: f32,
    _padding: vec2<f32>,
}                                                                                                                                                                                                                                                                               
                                                                                                                                                                                                                                                                                  
  @group(0) @binding(3) var<uniform> uniforms: Uniforms;  

@fragment
fn main(@location(0) tex_coord: vec2<f32>) -> @location(0) vec4<f32> {
    let scene = textureSample(scene_texture, tex_sampler, tex_coord).rgb;
    let bloom = textureSample(bloom_texture, tex_sampler, tex_coord).rgb;

    var hdr = scene + bloom * uniforms.bloom_intensity;

    let ldr_color = hdr / (hdr + vec3<f32>(1.0));

    return vec4<f32>(ldr_color, 1.0);
}
