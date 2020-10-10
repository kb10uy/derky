mod application;
mod rendering;
mod wavefront_obj;

use application::Application;
use std::{
    error::Error,
    time::{Duration, Instant},
};

use glium::{
    framebuffer::{MultiOutputFrameBuffer, SimpleFrameBuffer},
    glutin::{
        dpi::PhysicalSize,
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
        ContextBuilder,
    },
    texture::{DepthFormat, DepthTexture2d, MipmapsOption, Texture2d, UncompressedFloatFormat},
    uniform,
    uniforms::{EmptyUniforms, UniformsStorage},
    Display, Surface,
};
use log::info;
use ultraviolet::Mat4;

type AnyResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

/// 各種バッファの運搬用
struct Buffers {
    out_albedo: Texture2d,
    out_position: Texture2d,
    out_world_normal: Texture2d,
    lighting: Texture2d,
    depth: DepthTexture2d,
}

fn main() -> AnyResult<()> {
    pretty_env_logger::init();

    let (event_loop, display) = intialize_window();
    let mut app = Application::new(&display)?;

    // FrameBuffer セットアップ
    let fixed_buffers = initialize_buffers(&display)?;
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

    info!("Starting event loop");
    let mut last_at = Instant::now();
    event_loop.run(move |ev, _, control_flow| {
        let screen_matrix: [[f32; 4]; 4] = Mat4::identity().into();

        // delta time 計算
        let now = Instant::now();
        let delta = now - last_at;
        last_at = now;

        // tick 処理
        app.tick(delta);

        // ジオメトリパス
        let uniforms_generator = || EmptyUniforms;
        frame_buffer.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
        app.draw_geometry(&mut frame_buffer, uniforms_generator)
            .expect("Failed to process the geometry path");

        // ライティングパス
        lighting_buffer.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
        app.draw_lighting(
            &mut lighting_buffer,
            uniform! {
                env_screen_matrix: screen_matrix,
                g_position: &buffer_refs.out_position,
                g_normal: &buffer_refs.out_world_normal,
            },
        )
        .expect("Failed to process the lighting path");

        // 合成
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        app.draw_composition(
            &mut target,
            uniform! {
                env_screen_matrix: screen_matrix,
                tex_unlit: &buffer_refs.out_albedo,
                tex_lighting: &buffer_refs.lighting,
            },
        )
        .expect("Failed to process the composition path");
        target.finish().expect("Failed to finish drawing display");

        // ウィンドウイベント
        let next_frame = now + Duration::from_micros(16_666 / 2);
        *control_flow = ControlFlow::WaitUntil(next_frame);
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
    info!(
        "Supported OpenGL version: {}",
        display.get_opengl_version_string()
    );

    (event_loop, display)
}

fn initialize_buffers(display: &Display) -> AnyResult<Buffers> {
    let out_albedo = Texture2d::empty_with_format(
        display,
        UncompressedFloatFormat::F32F32F32F32,
        MipmapsOption::NoMipmap,
        1280,
        720,
    )?;
    let out_position = Texture2d::empty_with_format(
        display,
        UncompressedFloatFormat::F32F32F32F32,
        MipmapsOption::NoMipmap,
        1280,
        720,
    )?;
    let out_world_normal = Texture2d::empty_with_format(
        display,
        UncompressedFloatFormat::F32F32F32F32,
        MipmapsOption::NoMipmap,
        1280,
        720,
    )?;
    let lighting = Texture2d::empty_with_format(
        display,
        UncompressedFloatFormat::F32F32F32F32,
        MipmapsOption::NoMipmap,
        1280,
        720,
    )?;
    let depth = DepthTexture2d::empty_with_format(
        display,
        DepthFormat::F32,
        MipmapsOption::NoMipmap,
        1280,
        720,
    )?;

    Ok(Buffers {
        out_albedo,
        out_position,
        out_world_normal,
        lighting,
        depth,
    })
}
