use glam::{Vec2, Vec3};
use itertools::{EitherOrBoth, Itertools};
use wgpu::{util::DeviceExt, Device, Queue, RenderPipeline, SurfaceConfiguration, TextureView};

use crate::{
    camera::UniformCamera,
    primitives::{self, Material, ObjScene, Scene, UniformMaterial},
    texture, AppState, RenderStage,
};

#[derive(Debug)]
pub struct Geom {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    material_bind_group: wgpu::BindGroup,
    enable_bit: u32,
    enable_bit_buffer: wgpu::Buffer,
    model: ObjScene,
}

pub struct DefaultDebugRenderer {
    render_pipeline: RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    draw_count: u32,
}

impl DefaultDebugRenderer {
    pub fn new(
        device: &Device,
        config: &SurfaceConfiguration,
        _queue: &Queue,
        _state: &mut AppState,
        light_buffer: &wgpu::Buffer,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let (light_vertex, _) = ObjScene::load("cube/cube.obj", |_| false).unwrap();
        let draw_count: u32 = light_vertex[0].vertices().len() as u32;
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer: Light"),
            contents: bytemuck::cast_slice(&(light_vertex[0].vertices())),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer: Light"),
            contents: bytemuck::cast_slice(&(light_vertex[0].indices())),
            usage: wgpu::BufferUsages::INDEX,
        });
        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("Light Bind Group Layout"),
            });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: Some("Light Bind Group"),
        });
        let light_shader = device.create_shader_module(wgpu::include_wgsl!("light.wgsl"));
        let light_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Source Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
                push_constant_ranges: &[],
            });
        let light_vertex_descriptor = {
            use std::mem;
            wgpu::VertexBufferLayout {
                array_stride: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                }],
            }
        };
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Light Source Render Pipeline"),
            layout: Some(&light_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &light_shader,
                entry_point: Some("vs_main"),
                buffers: &[light_vertex_descriptor],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &light_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });
        Self {
            render_pipeline,
            vertex_buffer,
            index_buffer,
            bind_group,
            draw_count,
        }
    }

    fn render(&self, render_pass: &mut wgpu::RenderPass, camera_bind_group: &wgpu::BindGroup) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        render_pass.set_bind_group(1, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.draw_count, 0, 0..1);
    }
}

pub struct DefaultRenderer {
    render_pipeline: RenderPipeline,
    pub camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    pub light_buffer: wgpu::Buffer,
    scene_bind_group: wgpu::BindGroup,
    depth_texture: texture::Texture,
    debug_renderer: DefaultDebugRenderer,
    pub geoms: Vec<Geom>,
}

impl DefaultRenderer {
    pub fn new(
        device: &Device,
        config: &SurfaceConfiguration,
        queue: &Queue,
        state: &mut AppState,
        path: &str,
    ) -> Self {
        let mut geoms: Vec<Geom> = vec![];
        let (models, light) = primitives::ObjScene::load(path, |mt| mt.name == "Light").unwrap();
        state.given_light_position = light.is_some();
        // Scene light
        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: bytemuck::cast_slice::<_, u8>(&[Into::<primitives::UniformLight>::into(
                light.unwrap_or_else(|| Vec3::from(state.light_position)),
            )]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let scene_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("Scene Info Bind Group Layout"),
            });
        let scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &scene_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: Some("Camera Bind Group"),
        });
        // Setup Camera
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[UniformCamera::from_camera_project(
                &state.camera,
                &state.projection,
            )]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("Camera Bind Group Layout"),
            });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("Camera Bind Group"),
        });

        // Material Description
        let material_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // enable bit
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // color texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // normal texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("Material Bind Group Layout"),
            });

        // Depth buffer
        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        // Summon shader
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                    &material_bind_group_layout,
                    &scene_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[models
                    .iter()
                    .map(ObjScene::vertex_descriptor)
                    .next()
                    .unwrap()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: None,
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        for model in models {
            let (vertex_tangents, vertex_bitangents, vertex_normal) = model.tbn();
            let vertex_data = model
                .vertices()
                .iter()
                .zip(
                    model
                        .vertex_colors()
                        .iter()
                        .chain(std::iter::repeat(&Vec3::ONE)),
                )
                .zip(
                    model
                        .normals()
                        .iter()
                        .zip_longest(vertex_normal.iter())
                        .map(|z| match z {
                            EitherOrBoth::Both(l, _) => l,
                            EitherOrBoth::Left(l) => l,
                            EitherOrBoth::Right(r) => r,
                        })
                        .chain(std::iter::repeat(&Vec3::Z)),
                )
                .zip(vertex_tangents.iter().chain(std::iter::repeat(&Vec3::X)))
                .zip(vertex_bitangents.iter().chain(std::iter::repeat(&Vec3::Y)))
                .zip(
                    model
                        .texcoords()
                        .iter()
                        .chain(std::iter::repeat(&Vec2::ZERO)),
                )
                .flat_map(|(((((a, b), c), d), e), f)| {
                    a.to_array()
                        .into_iter()
                        .chain(b.to_array().into_iter())
                        .chain(c.to_array().into_iter())
                        .chain(d.to_array().into_iter())
                        .chain(e.to_array().into_iter())
                        .chain(f.to_array().into_iter())
                })
                .collect::<Box<[_]>>();
            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(format!("Vertex Buffer: {}", model.name()).as_str()),
                contents: bytemuck::cast_slice(&vertex_data),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(format!("Index Buffer: {}", model.name()).as_str()),
                contents: bytemuck::cast_slice(&model.indices()),
                usage: wgpu::BufferUsages::INDEX,
            });
            let (material_buffer, color_texture, normal_texture, enable_bit_buffer, enable_bit) = {
                let enable_bit_calc =
                    |color: bool, normal: bool| -> u32 { (color as u32) | ((normal as u32) << 1) };
                let unwrap_texture = |text: Option<texture::Texture>| -> texture::Texture {
                    text.unwrap_or(texture::Texture::empty(
                        &device,
                        &queue,
                        Some("Empty Texture"),
                    ))
                };
                if let Some(material) = model.material() {
                    let material_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some(format!("Material Buffer: {}", model.name()).as_str()),
                            contents: bytemuck::cast_slice(&[Into::<UniformMaterial>::into(
                                &material,
                            )]),
                            usage: wgpu::BufferUsages::UNIFORM,
                        });
                    let color_texture = material.color_texture.map(|img| {
                        texture::Texture::from_image(
                            &device,
                            &queue,
                            &img,
                            Some(format!("Color Texture: {}", model.name()).as_str()),
                        )
                        .unwrap()
                    });
                    let normal_texture = material.normal_texture.map(|img| {
                        texture::Texture::from_image_internal(
                            &device,
                            &queue,
                            &img,
                            Some(format!("Normal Texture: {}", model.name()).as_str()),
                            true,
                        )
                        .unwrap()
                    });
                    let enable_bit =
                        enable_bit_calc(color_texture.is_some(), normal_texture.is_some());
                    let enable_bit_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some(format!("Enable Bit Buffer: {}", model.name()).as_str()),
                            contents: bytemuck::cast_slice(&[enable_bit]),
                            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        });
                    (
                        material_buffer,
                        unwrap_texture(color_texture),
                        unwrap_texture(normal_texture),
                        enable_bit_buffer,
                        enable_bit,
                    )
                } else {
                    let material_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some(format!("Material Buffer: {}", model.name()).as_str()),
                            contents: bytemuck::cast_slice(&[Into::<UniformMaterial>::into(
                                Material::default(),
                            )]),
                            usage: wgpu::BufferUsages::UNIFORM,
                        });
                    let enable_bit_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some(format!("Enable Bit Buffer: {}", model.name()).as_str()),
                            contents: bytemuck::cast_slice(&[0u32]),
                            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        });
                    (
                        material_buffer,
                        unwrap_texture(None),
                        unwrap_texture(None),
                        enable_bit_buffer,
                        0u32,
                    )
                }
            };
            let material_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &material_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: material_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: enable_bit_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&color_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&color_texture.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(&normal_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
                    },
                ],
                label: Some(format!("Material Bind Group: {}", model.name()).as_str()),
            });
            geoms.push(Geom {
                vertex_buffer,
                index_buffer,
                material_bind_group,
                enable_bit,
                enable_bit_buffer,
                model,
            });
        }
        let debug_renderer = DefaultDebugRenderer::new(
            device,
            config,
            queue,
            state,
            &light_buffer,
            &camera_bind_group_layout,
        );
        Self {
            render_pipeline,
            camera_bind_group,
            camera_buffer,
            light_buffer,
            scene_bind_group,
            depth_texture,
            debug_renderer,
            geoms,
        }
    }
}

impl RenderStage<crate::AppState> for DefaultRenderer {
    fn render(
        &self,
        _state: &mut AppState,
        view: &TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass: everything"),
            color_attachments: &[
                // This is what @location(0) in the fragment shader targets
                Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                }),
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        render_pass.set_pipeline(&self.render_pipeline);
        for Geom {
            vertex_buffer,
            index_buffer,
            material_bind_group,
            model,
            ..
        } in &self.geoms
        {
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, material_bind_group, &[]);
            render_pass.set_bind_group(2, &self.scene_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..model.vertex_count(), 0, 0..1);
        }

        self.debug_renderer
            .render(&mut render_pass, &self.camera_bind_group);
    }

    fn resize(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        self.depth_texture =
            texture::Texture::create_depth_texture(device, config, "depth_texture");
    }

    fn update(&mut self, state: &crate::AppState, queue: &wgpu::Queue) {
        if state.normal_map_changed {
            for geom in &self.geoms {
                let enable_bit = geom.enable_bit & ((state.enable_normal_map as u32) << 1 | 1);
                queue.write_buffer(
                    &geom.enable_bit_buffer,
                    0,
                    bytemuck::cast_slice(&[enable_bit]),
                );
            }
        }
    }
}
