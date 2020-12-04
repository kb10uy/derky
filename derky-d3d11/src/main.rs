mod application;
mod rendering;

use crate::{
    application::{load_obj, ModelVertex, MODEL_VERTEX_LAYOUT},
    rendering::{
        create_d3d11, create_input_layout, create_viewport, load_pixel_shader, load_vertex_shader,
        ConstantBuffer, IndexBuffer, Topology, VertexBuffer, SCREEN_QUAD_INDICES,
        SCREEN_QUAD_VERTICES, VERTEX_INPUT_LAYOUT,
    },
};
use std::time::{Duration, Instant};

use anyhow::Result;
use log::info;
use ultraviolet::{projection::lh_yup::perspective_wgpu_dx, Mat4, Vec3};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::windows::WindowExtWindows,
    window::WindowBuilder,
};

#[derive(Debug)]
struct Matrices {
    model: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    projection: [[f32; 4]; 4],
}

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

    let (vs, vs_binary) = load_vertex_shader(&device, "derky-d3d11/shaders/geometry.vso")?;
    let ps = load_pixel_shader(&device, "derky-d3d11/shaders/geometry.pso")?;
    let input_layout = create_input_layout(&device, &MODEL_VERTEX_LAYOUT, &vs_binary)?;
    let model = load_obj(&device, "assets/Natsuki.obj")?;

    let mut matrices = Matrices {
        model: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)).into(),
        view: Mat4::look_at_lh(
            Vec3::new(0.0, 1.0, -2.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        )
        .into(),
        // なぜ near と far が逆になっているのかは謎
        projection: perspective_wgpu_dx(60f32.to_radians(), 16.0 / 9.0, 1024.0, 0.1).into(),
    };
    let constants = ConstantBuffer::new(&device, &matrices)?;

    // ------------------------------------------------------------------------
    info!("Starting event loop");
    let frame_time = Duration::from_nanos(33_333_333);
    let mut last_at = Instant::now();
    let started = Instant::now();
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

        matrices.model = Mat4::from_rotation_y(started.elapsed().as_secs_f32()).into();
        constants.update(&context, &matrices);

        let start = Instant::now();
        context.clear();
        context.set_viewport(&viewport);
        context.set_shaders(&input_layout, &vs, &ps);
        context.set_constant_buffer_vertex(0, &constants);

        for ((vb, ib), _texture) in model.visit() {
            context.set_vertices(&vb, &ib, Topology::Triangles);
            context.draw_with_indices(ib.len());
        }
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
