mod application;

use crate::application::Application;

use std::{
    slice::from_ref,
    time::{Duration, Instant},
};

use anyhow::Result;
use derky::d3d11::{
    buffer::{IndexBuffer, VertexBuffer},
    context::{create_d3d11, create_viewport},
    shader::{InputLayout, PixelShader, VertexShader},
    texture::{DepthStencil, Sampler},
    vertex::{Topology, SCREEN_QUAD_INDICES, SCREEN_QUAD_VERTICES, VERTEX_INPUT_LAYOUT},
};
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
    let viewport = create_viewport((1280, 720));

    let vs = VertexShader::load_object(&device, "assets/shaders/d3d11-compiled/screen.vso")?;
    let ps = PixelShader::load_object(&device, "assets/shaders/d3d11-compiled/screen.pso")?;
    let input_layout = InputLayout::create(&device, &VERTEX_INPUT_LAYOUT, &vs.binary())?;
    let screen_vb = VertexBuffer::new(&device, &SCREEN_QUAD_VERTICES)?;
    let screen_ib = IndexBuffer::new(&device, &SCREEN_QUAD_INDICES)?;
    let sampler = Sampler::new(&device)?;

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
        application.tick(&context, delta);
        application.draw_geometry(&context);

        render_target.clear(&context);
        depth_stencil.clear(&context);
        context.set_render_target(from_ref(&render_target), Some(&depth_stencil));
        context.set_viewport(&viewport);
        let g_buffers = application.g_buffer_textures();
        context.set_texture(0, Some(&g_buffers[0]));
        context.set_sampler(0, Some(&sampler));
        context.set_shaders(&input_layout, &vs, &ps);
        context.set_vertices(&screen_vb, &screen_ib, Topology::Triangles);
        context.draw_with_indices(screen_ib.len());

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
