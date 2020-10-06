//! レンダリング全般に利用される構造体などのモジュール。

use crate::AnyResult;
use std::{
    fs::File,
    io::{prelude::*, BufReader},
};

use glium::{
    backend::Facade,
    uniforms::{EmptyUniforms, UniformValue, Uniforms},
    Program,
};
use log::error;

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
pub fn load_program(display: &impl Facade, basename: &str) -> AnyResult<Program> {
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
