use std::process;
use wgpu_sandbox::App;

use log::error;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap_or_else(|err| {
        error!("failed to create event loop: {}", err);
        process::exit(1);
    });

    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap_or_else(|err| {
        error!("failed to run application: {}", err);
        process::exit(1);
    });
}
