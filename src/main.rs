mod framework;
mod wavefront_obj;

use std::time::{Duration, Instant};

use glium::{
    glutin::{
        dpi::PhysicalSize,
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
        ContextBuilder,
    },
    Display, Surface,
};
use log::info;

fn main() {
    pretty_env_logger::init();

    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_resizable(false)
        .with_inner_size(PhysicalSize::new(1280, 720));
    let cb = ContextBuilder::new();
    let display = Display::new(wb, cb, &event_loop).expect("Failed to create display");

    info!("Starting event loop");
    event_loop.run(move |ev, _, control_flow| {
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        target.finish().expect("Failed to finish drawing display");

        let next_frame_time = Instant::now() + Duration::from_secs_f64(16.66666e-3);
        *control_flow = ControlFlow::WaitUntil(next_frame_time);
        match ev {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
                _ => return,
            },
            _ => (),
        }
    });
}
