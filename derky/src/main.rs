mod application;
mod rendering;

use application::Application;
use rendering::Buffers;
use std::time::{Duration, Instant};

use anyhow::Result;
use glium::{
    buffer::{Buffer, BufferMode, BufferType},
    framebuffer::{MultiOutputFrameBuffer, SimpleFrameBuffer},
    glutin::{
        event::{Event, WindowEvent},
        event_loop::ControlFlow,
    },
    uniform, Surface,
};
use log::info;
use ultraviolet::Mat4;

struct Luminances {
    previous: Buffer<[u32]>,
    next: Buffer<[u32]>,
}

fn main() -> Result<()> {
    pretty_env_logger::init();

    let (event_loop, display) = rendering::intialize_window();
    let mut app = Application::new(&display)?;

    // FrameBuffer セットアップ
    let fixed_buffers = rendering::initialize_buffers(&display)?;
    let buffer_refs: &'static Buffers = unsafe { std::mem::transmute(&fixed_buffers) };
    let mut frame_buffer = MultiOutputFrameBuffer::with_depth_buffer(
        &display,
        vec![
            ("out_albedo", &buffer_refs.out_albedo),
            ("out_position", &buffer_refs.out_position),
            ("out_world_normal", &buffer_refs.out_world_normal),
        ],
        &buffer_refs.depth,
    )?;
    let mut lighting_buffer =
        SimpleFrameBuffer::with_depth_buffer(&display, &buffer_refs.lighting, &buffer_refs.depth)?;

    let mut next_luminance = Buffer::new(
        &display,
        &0u32,
        BufferType::AtomicCounterBuffer,
        BufferMode::Dynamic,
    )?;

    info!("Starting event loop");
    let frame_time = Duration::from_nanos(33_333_333);
    let mut last_at = Instant::now();
    event_loop.run(move |ev, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match ev {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => (),
            },
            _ => (),
        }

        let delta = last_at.elapsed();
        if delta < frame_time {
            return;
        } else {
            last_at = Instant::now();
        }

        let screen_matrix: [[f32; 4]; 4] = Mat4::identity().into();

        let prev_luminance = {
            let mut mapped = next_luminance.map();
            let prev_value = *mapped;
            *mapped = 0;
            prev_value
        } as f32;

        // tick 処理
        app.tick(delta);
        info!(
            "Delta: {:.2}ms, Luminance total: {:?}",
            delta.as_secs_f64() * 1000.0,
            prev_luminance
        );

        // ジオメトリパス
        let uniforms_generator = || {
            uniform! {}
        };
        frame_buffer.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
        app.draw_geometry(&mut frame_buffer, uniforms_generator)
            .expect("Failed to process the geometry path");

        // ライティングパス
        let uniforms_generator = || {
            uniform! {
                env_screen_matrix: screen_matrix,
                g_position: &buffer_refs.out_position,
                g_normal: &buffer_refs.out_world_normal,
            }
        };
        lighting_buffer.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
        app.draw_lighting(&mut lighting_buffer, uniforms_generator)
            .expect("Failed to process the lighting path");

        // コンポジション
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        app.draw_composition(
            &mut target,
            uniform! {
                next_luminance: &next_luminance,
                prev_luminance: prev_luminance,
                env_screen_matrix: screen_matrix,
                tex_unlit: &buffer_refs.out_albedo,
                tex_lighting: &buffer_refs.lighting,
            },
        )
        .expect("Failed to process the composition path");
        target.finish().expect("Failed to finish drawing display");
    });
}
