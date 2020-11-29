mod rendering;

use crate::rendering::{
    create_d3d11, create_input_layout, create_vertex_buffer, create_viewport, load_pixel_shader,
    load_vertex_shader, Texture, Topology, SCREEN_QUAD_VERTICES, VERTEX_LAYOUT,
};
use std::time::{Duration, Instant};

use anyhow::Result;
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
    let (device, context) = create_d3d11(window_handle, (1280, 720))?;
    let viewport = create_viewport((1280, 720));

    let (vs, vs_binary) = load_vertex_shader(&device, "derky-d3d11/shaders/geometry.vs.bin")?;
    let ps = load_pixel_shader(&device, "derky-d3d11/shaders/geometry.ps.bin")?;
    let input_layout = create_input_layout(&device, &VERTEX_LAYOUT, &vs_binary)?;
    let vertices = create_vertex_buffer(&device, &SCREEN_QUAD_VERTICES)?;

    // let texture = Texture::load_hdr(&device, "assets/background.exr")?;

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
        context.clear();
        context.set_viewport(&viewport);
        context.set_shaders(&input_layout, &vs, &ps);
        context.set_vertex_buffer(&vertices, Topology::Triangles);
        context.draw(SCREEN_QUAD_VERTICES.len());
        context.present();
        let process_time = start.elapsed();

        // TODO: 描画処理
        info!(
            "Delta: {:.2}ms, Elapsed: {:.3}ms",
            delta.as_secs_f32() * 1000.0,
            process_time.as_secs_f32() * 1000.0
        );
    });
}
