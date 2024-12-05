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
    @location(3) texcoord: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) texcoord: vec2<f32>,
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
var<uniform> light: Light;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = vec3<f32>(0.0, 0.0, 0.0);
    if material.ambient.w > 0 {
        color += material.ambient.xyz * 0.05;
    }

    let normal = normalize(in.normal);
    let direction = normalize(light.position - in.world_position);
    let nDotL = max(dot(direction, normal), 0.0);
    if material.diffuse.w > 0 {
        color += material.diffuse.xyz * 0.7 * nDotL;
    }
    if material.specular.w > 0 {
        let view_dir = normalize(camera.view_position.xyz - in.world_position);
        let reflect_dir = reflect(-direction, in.normal);
        let strength = pow(max(dot(view_dir, reflect_dir), 0.0), material.shininess);
        color += material.specular.xyz * strength * 1.0;
    }
    return vec4<f32>(color * in.color, 1.0);
}