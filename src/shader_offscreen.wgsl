// Vertex shader


@group(1) @binding(0)
var<uniform> camera: array<mat4x4<f32>, @view_count@>;
@group(1) @binding(1)
var<uniform> direction: vec4<f32>;

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
    //@location(6) view_index: i32,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera[model.view_index]* vec4<f32>(model.position, 1.0);
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
var radiance_cache: binding_array<texture_2d_array<f32>, 6>;
@group(0) @binding(1)
var radiance_sampler: binding_array<sampler, 6>;


fn true_radiance_sampler( direction: vec3<f32>, point_position: vec3<f32>) -> vec4<f32>{

    let probe_area_length = 10.0;
    let probe_inter_distance = 0.1;
    let probe_area_base_on_point = vec3( -5.0 + probe_inter_distance/2.0, -5.0 + probe_inter_distance/2.0, -5.0 + probe_inter_distance/2.0);
    let radiance_texture_coord = ( point_position - probe_area_base_on_point) / probe_inter_distance;
    /* maybe can be use in somewhere
    let xy1 = ( direction.x + direction.y > 0);
    let xy2 = ( direction.x - direction.y > 0);
    let yz1 = ( direction.y + direction.z > 0);
    let yz2 = ( direction.y - direction.z > 0);
    let zx1 = ( direction.z + direction.x > 0);
    let zx2 = ( direction.z - direction.x > 0);
    let xflag = ((!( xy1 ^ xy2)) && !( zx1 ^ !zx2)))
    let yflag = ((!( yz1 ^ yz2)) && !( xy1 ^ !xy2)));
    let zflag = !( xflag || yflag);
    let j = i32(xflag) * ((i32( sign(direction.x)) + 1) / 2)
                + f32(yflag) * ( 2 + (i32( sign(direction.y)) + 1) / 2)
                + f32(zflag) * ( 4 + (i32( sign(direction.z)) + 1) / 2);
    */
    //only for casecade 1
    let dir = normalize( vec3( direction.xyz));
    let idx_x = ( i32(sign(dir.x)) + 1) / 2;
    let idx_y = 2 + ( i32(sign(dir.y)) + 1) / 2;
    let idx_z = 4 + ( i32(sign(dir.z)) + 1) / 2;
    let ceil_coord = vec3<i32>( ceil( radiance_texture_coord));
    let floor_coord = vec3<i32>( floor( radiance_texture_coord));
    let remaind = radiance_texture_coord - vec3<f32>(floor_coord);
    let coord_on_plane = radiance_texture_coord / 255.0;
    let a = radiance_sampler[idx_x];
    let b = radiance_cache[idx_x];
    let color_A_plane_x_dir = textureSample(radiance_cache[idx_x], radiance_sampler[idx_x],
                            vec2( coord_on_plane.y, coord_on_plane.z ), ceil_coord.x);
    let color_B_plane_x_dir = textureSample(radiance_cache[idx_x], radiance_sampler[idx_x],
                            vec2( coord_on_plane.y, coord_on_plane.z ), floor_coord.x);
    let color_A_plane_y_dir = textureSample(radiance_cache[idx_y], radiance_sampler[idx_y],
                            vec2( coord_on_plane.x, coord_on_plane.z ), ceil_coord.y);
    let color_B_plane_y_dir = textureSample(radiance_cache[idx_y], radiance_sampler[idx_y],
                            vec2( coord_on_plane.x, coord_on_plane.z ), floor_coord.y);
    let color_A_plane_z_dir = textureSample(radiance_cache[idx_z], radiance_sampler[idx_z],
                            vec2( coord_on_plane.y, coord_on_plane.x ), ceil_coord.z);
    let color_B_plane_z_dir = textureSample(radiance_cache[idx_z], radiance_sampler[idx_z],
                            vec2( coord_on_plane.y, coord_on_plane.x ), floor_coord.z);
    let color_x_dir = (1.0 - remaind.x) * color_B_plane_x_dir + remaind.x * color_A_plane_x_dir;
    let color_y_dir = (1.0 - remaind.y) * color_B_plane_y_dir + remaind.y * color_A_plane_y_dir;
    let color_z_dir = (1.0 - remaind.z) * color_B_plane_z_dir + remaind.z * color_A_plane_z_dir;

    let total = abs(dir.x) + abs(dir.y) + abs(dir.z);
    return ( abs(dir.x) / total) * color_x_dir + ( abs(dir.y) / total) * color_y_dir + ( abs(dir.z) / total) * color_z_dir;
    //

}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texcoord = vec2<f32>(in.texcoord.x, 1.0 - in.texcoord.y);

    let color = (in.color * f32(~(enable_bit & 1) & 1)) + (textureSample(color_texture, color_sampler, texcoord).xyz * f32(enable_bit & 1));

    var light_color = vec3<f32>(0.0, 0.0, 0.0);
    light_color += material.ambient.xyz * 0.05 * material.ambient.w;

    let coef = (textureSample(normal_texture, normal_sampler, texcoord).xyz * 2 - 1);
    let raw_normal = (normalize(in.normal) * f32(((~(enable_bit & 2)) >> 1) & 1)) + (normalize(coef.x * normalize(in.tangent) + coef.y * normalize(in.bitangent) + coef.z * in.normal) * f32((enable_bit & 2) >> 1));
    let view_dir = -normalize(direction.xyz);
    let nDotV = dot(view_dir, raw_normal);
    let normal = f32(i32(nDotV < 1e-6) * -2 + 1 ) * raw_normal;
    let refection_dir = (abs( nDotV) * 2) * normal - view_dir;

    let light_from_probe = true_radiance_sampler( refection_dir, in.world_position);

    let nDotL = max(dot(refection_dir, normal), 0.0);
    light_color += material.diffuse.w * 0.7 * nDotL * light_from_probe.xyz;

    /*let half_dir = normalize(view_dir + vec3(0.0));
    let strength = pow(max(dot(in.normal, half_dir), 0.0), material.shininess);
    light_color += material.specular.xyz * strength * 1.0 * material.specular.w * f32(i32(nDotV > 1e-6));*/

    let pred = (material.ambient.xyz - vec3<f32>(1e-5)) + (material.diffuse.xyz - vec3<f32>(1e-5)) + (material.specular.xyz - vec3<f32>(1e-5));
    return vec4<f32>((light_color + f32((pred.x + pred.y + pred.z) <= 0)) * color, 1.0);
}