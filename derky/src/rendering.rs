//! レンダリング全般に利用される構造体などのモジュール。

use std::{
    fs::File,
    io::{prelude::*, BufReader},
};

use anyhow::Result;
use glium::{
    backend::Facade,
    glutin::{dpi::PhysicalSize, event_loop::EventLoop, window::WindowBuilder, ContextBuilder},
    texture::{DepthFormat, DepthTexture2d, MipmapsOption, Texture2d, UncompressedFloatFormat},
    uniforms::{EmptyUniforms, UniformValue, Uniforms},
    Display, Program,
};
use log::error;
use log::info;

/// 各種バッファの運搬用
pub struct Buffers {
    pub out_albedo: Texture2d,
    pub out_position: Texture2d,
    pub out_world_normal: Texture2d,
    pub lighting: Texture2d,
    pub depth: DepthTexture2d,
}

/// UniformsStorage を結合するやつ。
pub struct UniformsSet<H, T>(H, T);

impl<H: Uniforms> UniformsSet<H, EmptyUniforms> {
    /// UniformsStorage を食って UniformsSet にする。
    pub fn new(head: H) -> Self {
        UniformsSet(head, EmptyUniforms)
    }
}

impl<H: Uniforms, T: Uniforms> UniformsSet<H, T> {
    /// Uniforms を結合する。
    pub fn add<NH: Uniforms>(self, new_head: NH) -> UniformsSet<NH, UniformsSet<H, T>> {
        UniformsSet(new_head, self)
    }
}

impl<H: Uniforms, T: Uniforms> Uniforms for UniformsSet<H, T> {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut callback: F) {
        self.0.visit_values(&mut callback);
        self.1.visit_values(&mut callback);
    }
}

/// シェーダーを読み込む。
pub fn load_program(display: &impl Facade, basename: &str) -> Result<Program> {
    let mut vertex_file = BufReader::new(File::open(format!("shaders/{}.vert", basename))?);
    let mut fragment_file = BufReader::new(File::open(format!("shaders/{}.frag", basename))?);

    let mut vertex_shader = String::with_capacity(1024);
    let mut fragment_shader = String::with_capacity(1024);

    vertex_file.read_to_string(&mut vertex_shader)?;
    fragment_file.read_to_string(&mut fragment_shader)?;

    let program =
        Program::from_source(display, &vertex_shader, &fragment_shader, None).map_err(|e| {
            error!("Failed to compile the shader \"{}\": {}", basename, e);
            e
        })?;
    Ok(program)
}

/// シェーダーを読み込む。
pub fn load_screen_program(display: &impl Facade, basename: &str) -> Result<Program> {
    let mut vertex_file = BufReader::new(File::open("shaders/deferred_screen.vert")?);
    let mut fragment_file = BufReader::new(File::open(format!("shaders/{}.frag", basename))?);

    let mut vertex_shader = String::with_capacity(1024);
    let mut fragment_shader = String::with_capacity(1024);

    vertex_file.read_to_string(&mut vertex_shader)?;
    fragment_file.read_to_string(&mut fragment_shader)?;

    let program =
        Program::from_source(display, &vertex_shader, &fragment_shader, None).map_err(|e| {
            error!("Failed to compile the shader \"{}\": {}", basename, e);
            e
        })?;
    Ok(program)
}

/// ウィンドウを生成する。
pub fn intialize_window() -> (EventLoop<()>, Display) {
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

/// バッファを生成する。
pub fn initialize_buffers(display: &Display) -> Result<Buffers> {
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
