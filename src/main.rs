use include_dir::Dir;
use winit::event_loop::{ControlFlow, EventLoop};

// mod state;
mod app;
mod camera;
mod egui_tools;
mod renderer;
mod texture;
mod vertex;

static ASSETS_DIR: Dir<'_> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/resources");

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        pollster::block_on(run());
    }
}

async fn run() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = app::App::new();

    event_loop.run_app(&mut app).expect("Failed to run app");
}
