mod application;

use crate::application::Application;

use std::time::{Duration, Instant};

use anyhow::Result;
use derky::d3d11::{context::create_d3d11, texture::DepthStencil};
use log::info;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::windows::WindowExtWindows,
    window::WindowBuilder,
};

fn main() -> Result<()> {
    pretty_env_logger::init();

    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new()
        .with_title("Derky (Direct3D 11)")
        .with_inner_size(PhysicalSize::new(1280, 720))
        .with_resizable(false);
    let window = window_builder.build(&event_loop)?;
    let window_handle = window.hwnd();

    // ------------------------------------------------------------------------
    let (device, context, render_target) = create_d3d11(window_handle, (1280, 720))?;
    let depth_stencil = DepthStencil::create(&device, (1280, 720))?;
    let mut application = Application::new(&device)?;
    // ------------------------------------------------------------------------

    info!("Starting event loop");
    let frame_time = Duration::from_nanos(33_333_333);
    let mut last_at = Instant::now();
    event_loop.run(move |ev, _, flow| {
        match ev {
            // 終了
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *flow = ControlFlow::Exit,

            // その他
            _ => *flow = ControlFlow::Poll,
        }

        let delta = last_at.elapsed();
        if delta < frame_time {
            return;
        } else {
            last_at = Instant::now();
        }

        let start = Instant::now();
        // Measurement ------------------------------------

        // Tick process
        application.tick(&context, delta);

        // G-Buffer
        application.draw_geometry(&context);

        // Lighting Buffer
        application.draw_lighting(&context);

        // Composition
        application.draw_composition(&context, &render_target, &depth_stencil);

        // Present the Render Target
        context.present();

        // Measurement ------------------------------------
        let process_time = start.elapsed();

        // TODO: 描画処理
        info!(
            "Delta: {:.2}ms, Elapsed: {:.3}ms",
            delta.as_secs_f32() * 1000.0,
            process_time.as_secs_f32() * 1000.0
        );
    });
}
