use super::egui_tools::EguiRenderer;
use crate::camera::UniformCamera;
use crate::primitives::UniformLight;
use crate::renderer::DefaultRenderer;
use crate::{widget, AppState, RenderStage};
use egui_wgpu::{wgpu, ScreenDescriptor};
use glam::Vec3;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{
    DeviceEvent, DeviceId, ElementState, KeyEvent, MouseButton, MouseScrollDelta, TouchPhase,
    WindowEvent,
};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

pub struct AppInternal {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface<'static>,
    pub renderer: DefaultRenderer,
    pub egui_renderer: EguiRenderer,
    pub app_state: AppState,
}

impl AppInternal {
    async fn new(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        window: &Window,
        width: u32,
        height: u32,
    ) -> Self {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::TEXTURE_BINDING_ARRAY | wgpu::Features::MULTIVIEW,
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web, we'll have to disable some.
                    required_limits: wgpu::Limits::default(),
                    label: None,
                    memory_hints: Default::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();
        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let selected_format = wgpu::TextureFormat::Bgra8UnormSrgb;
        let swapchain_format = swapchain_capabilities
            .formats
            .iter()
            .find(|d| **d == selected_format)
            .expect("failed to select proper surface texture format!");

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: *swapchain_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 0,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &surface_config);

        let mut app_state = AppState::new();
        app_state
            .projection
            .resize(surface_config.width, surface_config.height);
        let egui_renderer = EguiRenderer::new(&device, surface_config.format, None, 1, window);
        let args: Vec<_> = std::env::args().collect();
        let renderer = DefaultRenderer::new(
            &device,
            &surface_config,
            &queue,
            &mut app_state,
            args.get(1).unwrap_or(&"cube/cube.obj".to_owned()),
        );

        Self {
            device,
            queue,
            surface,
            surface_config,
            egui_renderer,
            renderer,
            app_state,
        }
    }

    fn resize_surface(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
        self.app_state.projection.resize(width, height);
        self.renderer.resize(&self.device, &self.surface_config);
    }

    fn update(&mut self, dt: std::time::Duration) {
        self.app_state
            .camera_controller
            .update_camera(&mut self.app_state.camera, dt);
        self.queue.write_buffer(
            &self.renderer.camera_buffer,
            0,
            bytemuck::cast_slice(&[UniformCamera::from_camera_project(
                &self.app_state.camera,
                &self.app_state.projection,
            )]),
        );
        self.queue.write_buffer(
            &self.renderer.light_buffer,
            0,
            bytemuck::cast_slice(&[Into::<UniformLight>::into(Vec3::from(
                self.app_state.light_position,
            ))]),
        );
        self.renderer.update(&self.app_state, &self.queue);
    }

    fn keyboard_input(&mut self, event: &KeyEvent) -> bool {
        self.app_state.camera_controller.process_keyboard(
            &event.physical_key,
            &event.logical_key,
            event.state,
        );
        true
    }

    fn mouse_click(&mut self, state: ElementState, button: MouseButton) -> bool {
        if button == MouseButton::Left {
            self.app_state.mouse_pressed = state == ElementState::Pressed;
            true
        } else {
            false
        }
    }

    fn mouse_wheel(&mut self, delta: MouseScrollDelta, _phase: TouchPhase) -> bool {
        self.app_state.camera_controller.process_scroll(&delta);
        true
    }

    fn device_input(&mut self, event: &DeviceEvent) -> bool {
        if let DeviceEvent::MouseMotion { delta } = event {
            if self.app_state.mouse_pressed {
                self.app_state
                    .camera_controller
                    .process_mouse(delta.0, delta.1);
                return true;
            }
        }
        false
    }
}

pub struct App {
    instance: wgpu::Instance,
    last_render_time: std::time::Instant,
    state: Option<AppInternal>,
    window: Option<Arc<Window>>,
}

impl App {
    pub fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        Self {
            instance,
            state: None,
            window: None,
            last_render_time: std::time::Instant::now(),
        }
    }

    async fn set_window(&mut self, window: Window) {
        let window = Arc::new(window);
        let initial_width = 1360;
        let initial_height = 768;

        let _ = window.request_inner_size(PhysicalSize::new(initial_width, initial_height));

        let surface = self
            .instance
            .create_surface(window.clone())
            .expect("Failed to create surface!");

        let state = AppInternal::new(
            &self.instance,
            surface,
            &window,
            initial_width,
            initial_width,
        )
        .await;

        self.window.get_or_insert(window);
        self.state.get_or_insert(state);
    }

    fn handle_resized(&mut self, width: u32, height: u32) {
        self.state.as_mut().unwrap().resize_surface(width, height);
    }

    fn handle_redraw(&mut self, dt: std::time::Duration) {
        let state = self.state.as_mut().unwrap();
        state.update(dt);

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [state.surface_config.width, state.surface_config.height],
            pixels_per_point: self.window.as_ref().unwrap().scale_factor() as f32
                * state.app_state.scale_factor,
        };

        let surface_texture = state
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");

        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let window = self.window.as_ref().unwrap();

        state
            .renderer
            .render(&mut state.app_state, &surface_view, &mut encoder);

        {
            state.egui_renderer.begin_frame(window);

            widget::widget_show(&mut state.app_state, &state.egui_renderer);

            state.egui_renderer.end_frame_and_draw(
                &state.device,
                &state.queue,
                &mut encoder,
                window,
                &surface_view,
                screen_descriptor,
            );
        }
        state.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();
        pollster::block_on(self.set_window(window));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        // let egui render to process the event first
        self.state
            .as_mut()
            .unwrap()
            .egui_renderer
            .handle_input(self.window.as_ref().unwrap(), &event);

        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let _ = self.state.as_mut().unwrap().keyboard_input(&event);
            }
            WindowEvent::MouseWheel { delta, phase, .. } => {
                let _ = self.state.as_mut().unwrap().mouse_wheel(delta, phase);
            }
            WindowEvent::MouseInput { button, state, .. } => {
                let _ = self.state.as_mut().unwrap().mouse_click(state, button);
            }
            WindowEvent::RedrawRequested => {
                let now = std::time::Instant::now();
                let dt = now - self.last_render_time;
                self.last_render_time = now;
                self.handle_redraw(dt);

                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::Resized(new_size) => {
                self.handle_resized(new_size.width, new_size.height);
            }
            _ => (),
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let Some(state) = self.state.as_mut() {
            state.device_input(&event);
        }
    }
}
