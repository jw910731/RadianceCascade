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
    pub given_light_position: bool,
    pub light_position: [f32; 3],
    pub light_input: [String; 3],
}

impl AppState {
    pub fn new() -> Self {
        Self {
            scale_factor: 1.0,
            light_input: ["0.0".to_owned(), "0.0".to_owned(), "0.0".to_owned()],
            ..Default::default()
        }
    }
}
