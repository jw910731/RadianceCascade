use crate::camera;

pub trait RenderStage<T> {
    fn render(&self, state: &mut T, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder);
    fn resize(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration);
    fn update(&mut self, state: &T, queue: &wgpu::Queue);
}

#[derive(Debug, Clone, Default)]
pub struct AppState {
    pub camera: camera::Camera,
    pub projection: camera::Projection,
    pub camera_controller: camera::CameraController,
    pub mouse_pressed: bool,
    pub scale_factor: f32,
    pub enable_normal_map: bool,
    pub normal_map_changed: bool,
    pub given_light_position: bool,
    pub light_position: [f32; 3],
    pub light_input: [String; 3],
}

impl AppState {
    pub fn new() -> Self {
        let camera = camera::Camera::new((0.0, 5.0, 10.0), -90.0, -20.0);
        let projection = camera::Projection::new(1, 1, 45.0, 0.1, 100.0);
        let camera_controller = camera::CameraController::new(4.0, 0.4);
        Self {
            scale_factor: 1.0,
            light_input: ["0.0".to_owned(), "0.0".to_owned(), "0.0".to_owned()],
            enable_normal_map: true,
            camera,
            projection,
            camera_controller,
            ..Default::default()
        }
    }
}
