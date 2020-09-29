use std::{
    error::Error,
    fs::File,
    io::{prelude::*, BufReader},
};

use glium::{
    implement_vertex, index::PrimitiveType, uniform, Display, Frame, IndexBuffer, Program, Surface,
    VertexBuffer,
};
use ultraviolet::{projection::perspective_gl, Mat4};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub texture_uv: [f32; 2],
}
implement_vertex!(Vertex, position, normal, texture_uv);

pub struct Framework<'display> {
    display: &'display Display,
    view_matrix: Mat4,
    projection_matrix: Mat4,
}

impl<'display> Framework<'display> {
    pub fn new(display: &'display Display) -> Framework<'display> {
        let view_matrix = Mat4::identity();
        let projection_matrix = perspective_gl(60.0, 16.0 / 9.0, 0.001, 10.0);
        Framework {
            display,
            view_matrix,
            projection_matrix,
        }
    }

    pub fn load_program(&self, basename: &str) -> Result<Program, Box<dyn Error + Send + Sync>> {
        let mut vertex_file = BufReader::new(File::open(format!("shaders/{}.vert", basename))?);
        let mut fragment_file = BufReader::new(File::open(format!("shaders/{}.frag", basename))?);

        let mut vertex_shader = String::with_capacity(1024);
        let mut fragment_shader = String::with_capacity(1024);

        vertex_file.read_to_string(&mut vertex_shader)?;
        fragment_file.read_to_string(&mut fragment_shader)?;

        let program = Program::from_source(self.display, &vertex_shader, &fragment_shader, None)?;
        Ok(program)
    }

    pub fn draw_model(&self, target: &mut Frame, program: &Program) {
        let mat_model: [[f32; 4]; 4] = Mat4::identity().into();
        let mat_view: [[f32; 4]; 4] = self.view_matrix.into();
        let mat_projection: [[f32; 4]; 4] = self.projection_matrix.into();
        let uniforms = uniform! {
            mat_model: mat_model,
            mat_view: mat_view,
            mat_projection: mat_projection,
        };
        /*
        let vb = VertexBuffer::new(self.display, &vec![]).expect("Failed to create vertex buffer");
        let ib = IndexBuffer::new(self.display, PrimitiveType::TrianglesList, &vec![]);


        target
            .draw(&vb, &ib, program, &uniforms, &Default::default())
            .expect("Failed to draw");
        */
    }
}
