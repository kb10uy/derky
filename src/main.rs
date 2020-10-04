mod application;
mod environment;
mod model;
mod wavefront_obj;

use application::Application;
use std::{
    error::Error,
    time::{Duration, Instant},
};

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

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    pretty_env_logger::init();

    let (event_loop, display) = intialize_window();
    let mut app = Application::new(&display)?;

    info!("Starting event loop");
    let mut last_at = Instant::now();
    event_loop.run(move |ev, _, control_flow| {
        let now = Instant::now();
        let delta = now - last_at;
        last_at = now;

        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
        app.draw(&mut target, delta);
        target.finish().expect("Failed to finish drawing display");

        let next_frame_time = Instant::now() + Duration::from_nanos(16_666_666);
        *control_flow = ControlFlow::Poll;
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

fn intialize_window() -> (EventLoop<()>, Display) {
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_resizable(false)
        .with_inner_size(PhysicalSize::new(1280, 720));
    let cb = ContextBuilder::new();
    let display = Display::new(wb, cb, &event_loop).expect("Failed to create display");

    (event_loop, display)
}
