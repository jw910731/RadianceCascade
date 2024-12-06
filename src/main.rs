use winit::event_loop::{ControlFlow, EventLoop};

mod app;
mod camera;
mod primitives;
mod renderer;
mod texture;
mod widget;
mod window;
use app::*;

// static ASSETS_DIR: Dir<'_> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/resources");

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        pollster::block_on(run());
    }
}

async fn run() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = window::app::App::new();

    event_loop.run_app(&mut app).expect("Failed to run app");
}
