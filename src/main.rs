use winit::event_loop::{ControlFlow, EventLoop};

mod app;
mod camera;
mod primitives;
mod renderer;
mod texture;
mod widget;
mod window;
use app::*;

#[pollster::main]
async fn main() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = window::app::App::new();

    event_loop.run_app(&mut app).expect("Failed to run app");
}
