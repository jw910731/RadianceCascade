pub trait RenderStage<T> {
    fn render(&self, state: &mut T, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder);
    fn resize(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration);
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub scale_factor: f32,
}

impl Default for AppState {
    fn default() -> Self {
        Self { scale_factor: 1.0 }
    }
}
