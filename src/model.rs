//! 描画用モデルに関係するモジュール。

use crate::wavefront_obj::Group;
use std::error::Error;

use glium::{
    backend::Facade, implement_vertex, index::PrimitiveType, uniform, Display, Frame, IndexBuffer,
    Program, Surface, VertexBuffer,
};

/// 頂点シェーダーに渡る頂点情報を表す。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
}
implement_vertex!(Vertex, position, normal, uv);

/// VBO/IBO 化したモデルの情報を表す。
#[derive(Debug)]
pub struct Model {
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u32>,
}

impl Model {
    pub fn from_group(
        facade: &impl Facade,
        group: &Group,
    ) -> Result<Model, Box<dyn Error + Send + Sync>> {
        let mut vertices = vec![];
        let mut indices = vec![];

        for face in group.faces() {
            let vertex_base = vertices.len();
            let original_vertices: Vec<_> = face.collect();
            for original_vertice in &original_vertices {
                vertices.push(Vertex {
                    position: original_vertice.0.into(),
                    normal: original_vertice.2.unwrap_or_default().into(),
                    uv: original_vertice.1.unwrap_or_default().into(),
                });
            }
            for i in 0..(original_vertices.len() - 2) {
                indices.push(vertex_base as u32);
                indices.push((vertex_base + i + 1) as u32);
                indices.push((vertex_base + i + 2) as u32);
            }
        }

        let vertex_buffer = VertexBuffer::new(facade, &vertices)?;
        let index_buffer = IndexBuffer::new(facade, PrimitiveType::TrianglesList, &indices)?;

        Ok(Model {
            vertex_buffer,
            index_buffer,
        })
    }

    /// VBO を返す。
    pub fn vertex_buffer(&self) -> &VertexBuffer<Vertex> {
        &self.vertex_buffer
    }

    /// IBO を返す。
    pub fn index_buffer(&self) -> &IndexBuffer<u32> {
        &self.index_buffer
    }
}
