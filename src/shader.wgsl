// Vertex shader

struct Camera {
    view_matrix: mat4x4<f32>,
    view_position: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexInput {
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
    out.clip_position = camera.view_matrix * vec4<f32>(model.position, 1.0); 
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

@group(1) @binding(0)
var<uniform> material: Material;
@group(1) @binding(1)
var<uniform> enable_bit: u32;
@group(1) @binding(2)
var color_texture: texture_2d<f32>;
@group(1) @binding(3)
var color_sampler: sampler;
@group(1) @binding(4)
var normal_texture: texture_2d<f32>;
@group(1) @binding(5)
var normal_sampler: sampler;

@group(2) @binding(0)
var<uniform> light: Light;


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = in.color;
    let texcoord = vec2<f32>(in.texcoord.x, 1.0 - in.texcoord.y);
    if (enable_bit & 1) == 1 {
        color = textureSample(color_texture, color_sampler, texcoord).xyz;
    }
    var light_color = vec3<f32>(0.0, 0.0, 0.0);
    if material.ambient.w > 0 {
        light_color += material.ambient.xyz * 0.05;
    }

    var normal = normalize(in.normal);
    if (enable_bit & 2) == 2 {
            let coef = normalize(textureSample(normal_texture, normal_sampler, texcoord).xyz * 2 - 1);
            normal = normalize(coef.x * normalize(in.tangent) + coef.y * normalize(in.bitangent) + coef.z * in.normal);
    }

    let light_dir = normalize(light.position - in.world_position);
    let view_dir = normalize(camera.view_position.xyz - in.world_position);
    //let direction = normalize( camera.view_position.xyz - in.world_position);

    var nDotV = dot(view_dir, normal);
    if( nDotV < 1e-6){
        normal = normal * ( -1.0);
        nDotV *= -1.0;
    }
    let nDotL = dot(light_dir, normal);
    if( nDotL > 0.0){
        if material.diffuse.w > 0.0 {
            light_color += material.diffuse.xyz * 0.7 * nDotL;
        }
        if material.specular.w > 0.0 {
            let half_dir = normalize(view_dir + light_dir);
            let strength = pow(max(dot( normal, half_dir), 0.0), material.shininess);
            //let strength = pow(( nDotV + nDotL) / length( view_dir + light_dir), material.shininess);
            light_color += material.specular.xyz * strength * 1.0;
        }
    }

    return vec4<f32>(light_color * color, 1.0);
}