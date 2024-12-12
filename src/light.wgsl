// Vertex shader

struct Camera {
    view_matrix: mat4x4<f32>,
    view_position: vec4<f32>,
}

struct Light {
    position: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var<uniform> light: Light;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    let scale = mat4x4<f32>(0.1, 0.0, 0.0, 0.0,
                            0.0, 0.1, 0.0, 0.0,
                            0.0, 0.0, 0.1, 0.0,
                            0.0, 0.0, 0.0, 1.0);
    out.clip_position = camera.view_matrix * vec4<f32>((scale * vec4(model.position, 1.0)).xyz + light.position, 1.0);
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}