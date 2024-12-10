use std::{
    borrow::Borrow,
    path::{Path, PathBuf},
    sync::Arc,
};

use bytemuck::{NoUninit, Pod, Zeroable};
use glam::{vec2, vec3, Vec2, Vec3, Vec4};

// use crate::ASSETS_DIR;
const RESOURCE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/resources");

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable, Default)]
pub struct UniformLight {
    position: Vec4,
}

impl UniformLight {
    pub fn new(position: Vec4) -> Self {
        Self { position }
    }
}

impl<T> From<T> for UniformLight
where
    T: Borrow<Vec3>,
{
    fn from(value: T) -> Self {
        Self {
            position: (value.borrow().clone(), 1.0).into(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct UniformMaterial {
    // optional, alpha < 0 = None
    ambient: Vec4,
    diffuse: Vec4,
    specular: Vec4,
    shininess: f32,
    _padding: [u32; 3],
}

impl From<Option<Material>> for UniformMaterial {
    fn from(value: Option<Material>) -> Self {
        value.unwrap_or_else(|| Material::default()).into()
    }
}

impl<T> From<T> for UniformMaterial
where
    T: Borrow<Material>,
{
    fn from(value: T) -> Self {
        let op_vec3_to_vec4 = |v: Option<Vec3>| {
            Vec4::from((
                v.unwrap_or(vec3(0.0, 0.0, 0.0)),
                (2 * (v.is_some() as i32) - 1) as f32,
            ))
        };
        Self {
            ambient: op_vec3_to_vec4(value.borrow().ambient),
            diffuse: op_vec3_to_vec4(value.borrow().diffuse),
            specular: op_vec3_to_vec4(value.borrow().specular),
            shininess: value.borrow().shininess.unwrap_or(1.0),
            _padding: [0; 3],
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Material {
    pub ambient: Option<Vec3>,
    pub diffuse: Option<Vec3>,
    pub specular: Option<Vec3>,
    pub shininess: Option<f32>,
    pub color_texture: Option<image::DynamicImage>,
    pub normal_texture: Option<image::DynamicImage>,
}

pub trait Scene<V, C, N, T>
where
    V: NoUninit,
    C: NoUninit,
    N: NoUninit,
    T: NoUninit,
{
    fn vertex_descriptor(&self) -> wgpu::VertexBufferLayout<'static>;
    fn vertices(&self) -> Box<[V]>;
    fn vertex_colors(&self) -> Box<[C]>;
    fn normals(&self) -> Box<[N]>;
    fn tangent_bitangent(&self) -> (Box<[Vec3]>, Box<[Vec3]>);
    fn texcoords(&self) -> Box<[T]>;
    fn indices(&self) -> Box<[u32]>;
    fn vertex_count(&self) -> u32;
    fn name(&self) -> &str;
    fn material(&self) -> Option<Material>;
}

fn load_obj<P: AsRef<Path>>(obj_path: P) -> tobj::LoadResult {
    tobj::load_obj(
        PathBuf::from(RESOURCE_PATH).join(obj_path),
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
    )
}

#[derive(Debug, Clone)]
pub struct ObjScene {
    pub model: tobj::Model,
    pub obj_dir: PathBuf,
    pub materials: Option<Arc<tobj::Material>>,
}

impl ObjScene {
    pub fn load<P, F>(
        path: P,
        light_predicate: F,
    ) -> Result<(Vec<Self>, Option<Vec3>), tobj::LoadError>
    where
        P: AsRef<Path>,
        F: Fn(&tobj::Material) -> bool,
    {
        let (model, materials) = load_obj(&path)?;
        let materials = materials?.into_iter().map(Arc::new).collect::<Box<[_]>>();
        let light = model
            .iter()
            .filter_map(|md| {
                md.mesh
                    .material_id
                    .and_then(|i| materials.get(i).map(|m| m.clone()))
                    .take()
                    .map(|mat| (md, mat))
            })
            .filter(|(_, mt)| light_predicate(mt))
            // find position average point of the light object
            .map(|(md, _)| {
                md.mesh
                    .positions
                    .chunks(3)
                    .into_iter()
                    .map(Vec3::from_slice)
                    .sum::<Vec3>()
                    / ((md.mesh.positions.len() / 3) as f32)
            })
            // only one light is supported now
            .take(1)
            .next();
        Ok((
            model
                .into_iter()
                .map(|m| {
                    let material_id = m.mesh.material_id;
                    Self {
                        model: m,
                        obj_dir: PathBuf::from(RESOURCE_PATH)
                            .join(path.as_ref())
                            .parent()
                            .map(Path::to_path_buf)
                            .unwrap_or(RESOURCE_PATH.into()),
                        materials: material_id.and_then(|i| materials.get(i).map(Clone::clone)),
                    }
                })
                .collect(),
            light,
        ))
    }
}

impl Scene<Vec3, Vec3, Vec3, Vec2> for ObjScene {
    fn vertex_descriptor(&self) -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<[f32; 17]>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 9]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 15]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }

    fn vertices(&self) -> Box<[Vec3]> {
        self.model
            .mesh
            .positions
            .chunks(3)
            .map(|s| vec3(s[0], s[1], s[2]))
            .collect()
    }

    fn vertex_colors(&self) -> Box<[Vec3]> {
        self.model
            .mesh
            .vertex_color
            .chunks(3)
            .map(Vec3::from_slice)
            .collect()
    }

    fn normals(&self) -> Box<[Vec3]> {
        self.model
            .mesh
            .normals
            .chunks(3)
            .map(Vec3::from_slice)
            .collect()
    }

    fn tangent_bitangent(&self) -> (Box<[Vec3]>, Box<[Vec3]>) {
        let temp_vertices = self.vertices();
        let temp_texcoords = {
            let mut texcoords = self.texcoords();
            if texcoords.len() != temp_vertices.len() {
                texcoords = std::iter::repeat_n(Vec2::ZERO, temp_vertices.len()).collect();
            }
            texcoords
        };
        assert!(temp_vertices.len() == temp_texcoords.len());
        let mut temp_tangents = vec![Vec3::ZERO; temp_vertices.len()];
        let mut temp_bitangents = vec![Vec3::ZERO; temp_vertices.len()];
        let mut count_triangles_included = vec![0; temp_vertices.len()];
        for c in self.indices().chunks(3) {
            let pos0 = temp_vertices[c[0] as usize];
            let pos1 = temp_vertices[c[1] as usize];
            let pos2 = temp_vertices[c[2] as usize];

            let uv0: Vec2 = temp_texcoords[c[0] as usize];
            let uv1: Vec2 = temp_texcoords[c[1] as usize];
            let uv2: Vec2 = temp_texcoords[c[2] as usize];

            // Calculate the edges of the triangle
            let delta_pos1 = pos1 - pos0;
            let delta_pos2 = pos2 - pos0;

            // This will give us a direction to calculate the
            // tangent and bitangent
            let delta_uv1 = uv1 - uv0;
            let delta_uv2 = uv2 - uv0;

            // Solving the following system of equations will
            // give us the tangent and bitangent.
            //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
            //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
            // Luckily, the place I found this equation provided
            // the solution!
            let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
            let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
            // We flip the bitangent to enable right-handed normal
            // maps with wgpu texture coordinate system
            let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * r;

            // We'll use the same tangent/bitangent for each vertex in the triangle
            temp_tangents[c[0] as usize] = tangent + temp_tangents[c[0] as usize];
            temp_tangents[c[1] as usize] = tangent + temp_tangents[c[1] as usize];
            temp_tangents[c[2] as usize] = tangent + temp_tangents[c[2] as usize];
            temp_bitangents[c[0] as usize] = bitangent + temp_bitangents[c[0] as usize];
            temp_bitangents[c[1] as usize] = bitangent + temp_bitangents[c[1] as usize];
            temp_bitangents[c[2] as usize] = bitangent + temp_bitangents[c[2] as usize];
            // Used to average the tangents/bitangents
            count_triangles_included[c[0] as usize] += 1;
            count_triangles_included[c[1] as usize] += 1;
            count_triangles_included[c[2] as usize] += 1;
        }

        (
            temp_tangents
                .iter()
                .zip(count_triangles_included.iter())
                .map(|(tangent, count)| {
                    if *count > 0 {
                        tangent / (*count as f32)
                    } else {
                        Vec3::X
                    }
                })
                .collect(),
            temp_bitangents
                .iter()
                .zip(count_triangles_included.iter())
                .map(|(bitangent, count)| {
                    if *count > 0 {
                        bitangent / (*count as f32)
                    } else {
                        Vec3::Y
                    }
                })
                .collect(),
        )
    }

    fn texcoords(&self) -> Box<[Vec2]> {
        self.model
            .mesh
            .texcoords
            .chunks(2)
            .map(|s| vec2(s[0], s[1]))
            .collect()
    }

    fn indices(&self) -> Box<[u32]> {
        self.model
            .mesh
            .indices
            .chunks(3)
            .flat_map(|e| e.iter().cloned().rev())
            .collect::<Box<[_]>>()
    }

    fn vertex_count(&self) -> u32 {
        self.model.mesh.indices.len() as u32
    }

    fn name(&self) -> &str {
        &self.model.name
    }

    fn material(&self) -> Option<Material> {
        self.materials.as_ref().map(|e| {
            let color_texture = {
                let path = e.diffuse_texture.clone().map(|dp| self.obj_dir.join(dp));
                path.and_then(|p| {
                    image::ImageReader::open(p)
                        .ok()
                        .and_then(|img| img.decode().ok())
                })
            };
            let normal_texture = {
                let path = e.normal_texture.clone().map(|dp| self.obj_dir.join(dp));
                path.and_then(|p| {
                    image::ImageReader::open(p)
                        .ok()
                        .and_then(|img| img.decode().ok())
                })
            };
            Material {
                ambient: e.ambient.map(Vec3::from_array),
                diffuse: e.diffuse.map(Vec3::from_array),
                specular: e.specular.map(Vec3::from_array),
                shininess: e.shininess,
                color_texture,
                normal_texture,
            }
        })
    }
}
