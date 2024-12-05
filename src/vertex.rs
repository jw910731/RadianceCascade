use std::{borrow::Borrow, path::Path, sync::Arc};

use bytemuck::{NoUninit, Pod, Zeroable};
use glam::{vec2, vec3, Vec2, Vec3, Vec4};

use crate::ASSETS_DIR;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable, Default)]
pub struct UnifromLight {
    position: Vec4,
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
        value
            .unwrap_or_else(|| Material {
                ambient: None,
                diffuse: None,
                specular: None,
                shininess: None,
            })
            .into()
    }
}

impl<T> From<T> for UniformMaterial
where
    T: Borrow<Material>,
{
    fn from(value: T) -> Self {
        let op_vec3_to_vec4 = |v: Option<Vec3>| {
            Vec4::from((v.unwrap_or(vec3(0.0, 0.0, 0.0)), v.is_some() as i32 as f32))
        };
        Self {
            ambient: op_vec3_to_vec4(value.borrow().ambient),
            diffuse: op_vec3_to_vec4(value.borrow().diffuse),
            specular: op_vec3_to_vec4(value.borrow().specular),
            shininess: value.borrow().shininess.unwrap_or(-1.0),
            _padding: [0; 3],
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Material {
    pub ambient: Option<Vec3>,
    pub diffuse: Option<Vec3>,
    pub specular: Option<Vec3>,
    pub shininess: Option<f32>,
}

impl<T> From<T> for Material
where
    T: Borrow<tobj::Material>,
{
    fn from(value: T) -> Self {
        Self {
            ambient: value.borrow().ambient.map(Vec3::from_array),
            diffuse: value.borrow().diffuse.map(Vec3::from_array),
            specular: value.borrow().specular.map(Vec3::from_array),
            shininess: value.borrow().shininess,
        }
    }
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
    fn texcoords(&self) -> Box<[T]>;
    fn indices(&self) -> Box<[u32]>;
    fn vertex_count(&self) -> u32;
    fn name(&self) -> &str;
    fn material(&self) -> Option<Material>;
    fn light(&self) -> UnifromLight;
}

fn load_obj<P: AsRef<Path>>(obj_path: P) -> tobj::LoadResult {
    let obj_file = ASSETS_DIR
        .get_file(&obj_path)
        .ok_or(tobj::LoadError::OpenFileFailed)?;
    tobj::load_obj_buf(
        &mut std::io::Cursor::new(obj_file.contents()),
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| {
            let mtl_path = p
                .strip_prefix(obj_path.as_ref().parent().unwrap_or(Path::new("")))
                .or(Err(tobj::LoadError::OpenFileFailed))?;
            let mtl_file = ASSETS_DIR
                .get_file(mtl_path)
                .ok_or(tobj::LoadError::OpenFileFailed)?;
            tobj::load_mtl_buf(&mut std::io::Cursor::new(mtl_file.contents()))
        },
    )
}

#[derive(Debug, Clone)]
pub struct ObjScene {
    pub model: tobj::Model,
    pub materials: Option<Arc<tobj::Material>>,
    pub light: Vec3,
}

impl ObjScene {
    pub fn load<P, F>(path: P, light_predicate: F) -> Result<Vec<Self>, tobj::LoadError>
    where
        P: AsRef<Path>,
        F: Fn(&tobj::Material) -> bool,
    {
        let (model, materials) = load_obj(path)?;
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
            .inspect(|(md, _)| {
                dbg!(md
                    .mesh
                    .positions
                    .chunks(3)
                    .into_iter()
                    .map(Vec3::from_slice)
                    .collect::<Box<[_]>>());
            })
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
            .next()
            .unwrap_or(Vec3::ZERO);
        dbg!(light);
        Ok(model
            .into_iter()
            .map(|m| {
                let material_id = m.mesh.material_id;
                Self {
                    model: m,
                    materials: material_id.and_then(|i| materials.get(i).map(Clone::clone)),
                    light,
                }
            })
            .collect())
    }
}

impl Scene<Vec3, Vec3, Vec3, Vec2> for ObjScene {
    fn vertex_descriptor(&self) -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
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
        self.materials.as_ref().map(|e| e.clone().into())
    }

    fn light(&self) -> UnifromLight {
        UnifromLight {
            position: Into::into((self.light, 1.0)),
        }
    }
}
