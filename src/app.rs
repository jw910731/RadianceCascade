use glam::Vec3;

use crate::camera;

pub trait RenderStage<T> {
    fn render(&self, state: &mut T, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder);
    fn resize(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration);
}

#[derive(Debug, Clone, Default)]
pub struct AppState {
    pub camera: camera::Camera,
    pub scale_factor: f32,
    pub look_at_y: f32,
    pub eye_pos_rotation: f32,
    pub given_light_position: bool,
    pub light_position: [f32; 3],
    pub light_input: [String; 3],
}

impl AppState {
    pub fn new() -> Self {
        let camera = camera::Camera::new(
            // position the camera 1 unit up and 2 units back
            // +z is out of the screen
            (0.0, 3.0, 12.0).into(),
            // have it look at the origin
            (0.0, 2.0, 0.0).into(),
            Vec3::Y,
            1.0,
            45.0,
            0.1,
            100.0,
        );
        Self {
            scale_factor: 1.0,
            light_input: ["0.0".to_owned(), "0.0".to_owned(), "0.0".to_owned()],
            camera,
            ..Default::default()
        }
    }
}
