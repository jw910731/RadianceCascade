use std::default;

pub trait RenderStage<T> {
    fn render(&self, state: &mut T, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder);
    fn resize(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration);
}

#[derive(Debug, Clone, Default)]
pub struct AppState {
    pub scale_factor: f32,
    pub look_at_y: f32,
    pub eye_pos_rotation: f32,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            scale_factor: 1.0,
            ..Default::default()
        }
    }
}
