use glam::{vec3, Vec3};
use wgpu::{
    util::DeviceExt, CommandEncoder, Device, Queue, RenderPipeline, SurfaceConfiguration,
    TextureView,
};

use crate::{
    camera, texture,
    vertex::{self, ObjScene, Scene},
};

pub trait RenderStage {
    fn render(&self, view: &TextureView, encoder: &wgpu::Device) -> Vec<wgpu::CommandEncoder>;
    fn resize(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration);
}

#[derive(Debug)]
struct Geom {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    model: ObjScene,
}

pub struct DefaultRenderer {
    render_pipeline: RenderPipeline,
    pub camera: camera::Camera,
    pub camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    depth_texture: texture::Texture,
    geoms: Vec<Geom>,
}

impl DefaultRenderer {
    pub fn new(device: &Device, config: &SurfaceConfiguration, queue: &Queue) -> Self {
        let mut geoms: Vec<Geom> = vec![];
        let models = vertex::ObjScene::load("cornell-box.obj").unwrap();
        // Setup Camera
        let camera = camera::Camera::new(
            // position the camera 1 unit up and 2 units back
            // +z is out of the screen
            (0.0, 2.0, 2.0).into(),
            // have it look at the origin
            (0.0, 0.1, 0.0).into(),
            Vec3::Y,
            config.width as f32 / config.height as f32,
            45.0,
            0.1,
            100.0,
            0.2,
        );
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera.get_view_project()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        // Depth buffer
        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        // Summon shader
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
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
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
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
            let vertex_data = model
                .vertices()
                .into_iter()
                .zip(std::iter::repeat(&vec3(1.0, 0.0, 0.0)))
                .flat_map(|(a, b)| [*a, *b])
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
            geoms.push(Geom {
                vertex_buffer,
                index_buffer,
                model,
            });
        }
        Self {
            render_pipeline,
            camera,
            camera_bind_group,
            camera_buffer,
            depth_texture,
            geoms,
        }
    }
}

impl RenderStage for DefaultRenderer {
    fn render(&self, view: &TextureView, device: &wgpu::Device) -> Vec<wgpu::CommandEncoder> {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
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
                            a: 0.0,
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
        for Geom {
            vertex_buffer,
            index_buffer,
            model,
        } in &self.geoms
        {
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..model.vertex_count(), 0, 0..1);
        }
        vec![encoder]
    }

    fn resize(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        self.depth_texture =
            texture::Texture::create_depth_texture(device, config, "depth_texture");
    }
}
