use std::{path::Path, sync::Arc};

use bytemuck::NoUninit;
use glam::{vec2, vec3, Vec2, Vec3};

use crate::ASSETS_DIR;

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
    model: tobj::Model,
    materials: Arc<Vec<tobj::Material>>,
}

impl ObjScene {
    pub fn new(model: tobj::Model, materials: Vec<tobj::Material>) -> Self {
        Self {
            model,
            materials: Arc::new(materials),
        }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Vec<Self>, tobj::LoadError> {
        let (model, materials) = load_obj(path)?;
        let materials = Arc::new(materials?);
        Ok(model
            .into_iter()
            .map(|m| Self {
                model: m,
                materials: materials.clone(),
            })
            .collect())
    }
}

impl Scene<Vec3, Vec3, Vec3, Vec2> for ObjScene {
    fn vertex_descriptor(&self) -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<Vec3>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // wgpu::VertexAttribute {
                //     offset: mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                //     shader_location: 2,
                //     format: wgpu::VertexFormat::Float32x3,
                // },
                // wgpu::VertexAttribute {
                //     offset: mem::size_of::<[f32; 9]>() as wgpu::BufferAddress,
                //     shader_location: 3,
                //     format: wgpu::VertexFormat::Float32x2,
                // },
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
            .map(|s| vec3(s[0], s[1], s[2]))
            .collect()
    }

    fn normals(&self) -> Box<[Vec3]> {
        self.model
            .mesh
            .normals
            .chunks(3)
            .map(|s| vec3(s[0], s[1], s[2]))
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
            .flat_map(|n| [n[2], n[1], n[0]])
            .collect::<Box<[_]>>()
    }

    fn vertex_count(&self) -> u32 {
        self.model.mesh.indices.len() as u32
    }

    fn name(&self) -> &str {
        &self.model.name
    }
}
