mod application;
mod rendering;

use application::Application;
use rendering::{load_screen_program, Buffers, SCREEN_QUAD_INDICES, SCREEN_QUAD_VERTICES};
use std::time::{Duration, Instant};

use anyhow::Result;
use glium::{
    buffer::{Buffer, BufferMode, BufferType},
    framebuffer::{MultiOutputFrameBuffer, SimpleFrameBuffer},
    glutin::{
        event::{Event, WindowEvent},
        event_loop::ControlFlow,
    },
    index::PrimitiveType,
    uniform, DrawParameters, IndexBuffer, Surface, VertexBuffer,
};
use log::info;
use ultraviolet::Mat4;

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
    let mut luminance_buffers = vec![
        SimpleFrameBuffer::new(&display, &buffer_refs.luminance_first)?,
        SimpleFrameBuffer::new(&display, &buffer_refs.luminance_second)?,
    ];
    let mut luminance_textures = vec![&buffer_refs.luminance_first, &buffer_refs.luminance_second];

    let luminance_scaler_program = load_screen_program(&display, "deferred_scale_step")?;

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

        /*
        // 平均輝度を求めるやーつ
        let mut vertices_source = SCREEN_QUAD_VERTICES.clone();
        let indices =
            IndexBuffer::new(&display, PrimitiveType::TrianglesList, &SCREEN_QUAD_INDICES).unwrap();
        let params = DrawParameters::default();

        // 初回コピー
        let vertices = VertexBuffer::new(&display, &vertices_source).unwrap();
        let uniforms = uniform! {
            env_screen_matrix: screen_matrix,
            scale_prev_texture: &buffer_refs.out_albedo,
            scale_prev_size: (1280f32, 720f32),
        };
        luminance_buffers[0]
            .draw(
                &vertices,
                &indices,
                &luminance_scaler_program,
                &uniforms,
                &params,
            )
            .unwrap();

        // 1px になるまで繰り返す
        let mut prev_scale = 1024f32;
        while prev_scale > 1.0 {
            vertices_source.iter_mut().for_each(|v| {
                v.position[0] = (v.position[0] + 1.0) / 4.0 - 1.0;
                v.position[1] = (v.position[1] - 1.0) / 4.0 + 1.0;
            });
            let vertices =
                VertexBuffer::new(&display, &vertices_source).expect("Invalid vertex buffer");
            let uniforms = uniform! {
                env_screen_matrix: screen_matrix,
                scale_prev_texture: luminance_textures[0],
                scale_prev_size: (prev_scale, prev_scale),
            };

            luminance_buffers[1]
                .draw(
                    &vertices,
                    &indices,
                    &luminance_scaler_program,
                    &uniforms,
                    &params,
                )
                .expect("Failed to calculate luminance");

            luminance_buffers.swap(0, 1);
            luminance_textures.swap(0, 1);
            vertices_source.iter_mut().for_each(|v| {
                v.uv[0] /= 4.0;
                v.uv[1] /= 4.0;
            });
            prev_scale /= 4.0;
        }
        */

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
