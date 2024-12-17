// Vertex shader

@group(1) @binding(0)
var<uniform> camera: array<mat4x4<f32>, @view_count@>;

struct VertexInput {
    @builtin(view_index) view_index: i32,
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
    @location(5) texcoord: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
    @location(5) texcoord: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera[model.view_index] * vec4<f32>(model.position, 1.0); 
    out.world_position = model.position;
    out.color = model.color;
    out.normal = model.normal;
    out.texcoord = model.texcoord;
    out.tangent = model.tangent;
    out.bitangent = model.bitangent;
    return out;
}

// Fragment shader

struct Material {
    ambient: vec4<f32>,
    diffuse: vec4<f32>,
    specular: vec4<f32>,
    shininess: f32,
    // _padding: array<u32, 3>,
}

struct Light {
    position: vec3<f32>,
}

@group(2) @binding(0)
var<uniform> material: Material;
@group(2) @binding(1)
var<uniform> enable_bit: u32;
@group(2) @binding(2)
var color_texture: texture_2d<f32>;
@group(2) @binding(3)
var color_sampler: sampler;
@group(2) @binding(4)
var normal_texture: texture_2d<f32>;
@group(2) @binding(5)
var normal_sampler: sampler;

@group(0) @binding(0)
var radiance_cache: texture_2d_array<f32>;
@group(0) @binding(1)
var sample: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texcoord = vec2<f32>(in.texcoord.x, 1.0 - in.texcoord.y);

    let color = (in.color * f32(~(enable_bit & 1) & 1)) + (textureSample(color_texture, color_sampler, texcoord).xyz * f32(enable_bit & 1));

    var light_color = vec3<f32>(0.0, 0.0, 0.0);
    light_color += material.ambient.xyz * 0.05 * material.ambient.w;

    let coef = (textureSample(normal_texture, normal_sampler, texcoord).xyz * 2 - 1);
    let raw_normal = (normalize(in.normal) * f32(((~(enable_bit & 2)) >> 1) & 1)) + (normalize(coef.x * normalize(in.tangent) + coef.y * normalize(in.bitangent) + coef.z * in.normal) * f32((enable_bit & 2) >> 1));
    let view_dir = normalize(vec3<f32>(0.0) - in.world_position);
    let nDotV = dot(view_dir, raw_normal);
    let normal = f32(i32(nDotV < 1e-6) * -2 + 1 ) * raw_normal;

    let direction = normalize(vec3(0.0) - in.world_position);
    let nDotL = max(dot(direction, normal), 0.0);
    light_color += material.diffuse.xyz * 0.7 * nDotL * material.diffuse.w;

    let half_dir = normalize(view_dir + vec3(0.0));
    let strength = pow(max(dot(in.normal, half_dir), 0.0), material.shininess);
    light_color += material.specular.xyz * strength * 1.0 * material.specular.w * f32(i32(nDotV > 1e-6));

    let pred = (material.ambient.xyz - vec3<f32>(1e-5)) + (material.diffuse.xyz - vec3<f32>(1e-5)) + (material.specular.xyz - vec3<f32>(1e-5));
    return vec4<f32>((light_color + f32((pred.x + pred.y + pred.z) <= 0)) * color, 1.0);
}