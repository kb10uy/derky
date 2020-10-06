//! 描画用モデルに関係するモジュール。

use crate::{
    wavefront_obj::{Group, Object, WavefrontObj},
    AnyResult,
};
use std::error::Error;

use glium::{backend::Facade, implement_vertex, index::PrimitiveType, IndexBuffer, VertexBuffer};
use log::info;
use ultraviolet::Vec3;

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

#[allow(dead_code)]
impl Model {
    pub fn from_obj(facade: &impl Facade, obj: &WavefrontObj) -> AnyResult<Model> {
        let mut vertices = vec![];
        let mut indices = vec![];

        for object in obj.objects() {
            info!("Loading {:?}", object.name());
            for group in object.groups() {
                Model::convert_group(group, &mut vertices, &mut indices);
            }
        }

        Model::from_buffers(facade, &vertices, &indices)
    }

    pub fn from_object(facade: &impl Facade, object: &Object) -> AnyResult<Model> {
        let mut vertices = vec![];
        let mut indices = vec![];

        for group in object.groups() {
            Model::convert_group(group, &mut vertices, &mut indices);
        }

        Model::from_buffers(facade, &vertices, &indices)
    }

    pub fn from_group(
        facade: &impl Facade,
        group: &Group,
    ) -> Result<Model, Box<dyn Error + Send + Sync>> {
        let mut vertices = vec![];
        let mut indices = vec![];

        Model::convert_group(group, &mut vertices, &mut indices);

        Model::from_buffers(facade, &vertices, &indices)
    }

    /// VBO を返す。
    pub fn vertex_buffer(&self) -> &VertexBuffer<Vertex> {
        &self.vertex_buffer
    }

    /// IBO を返す。
    pub fn index_buffer(&self) -> &IndexBuffer<u32> {
        &self.index_buffer
    }

    fn from_buffers(
        facade: &impl Facade,
        vertices: &[Vertex],
        indices: &[u32],
    ) -> AnyResult<Model> {
        let vertex_buffer = VertexBuffer::new(facade, &vertices)?;
        let index_buffer = IndexBuffer::new(facade, PrimitiveType::TrianglesList, &indices)?;

        info!("Wavefront OBJ loaded; {} vertices", vertices.len());

        Ok(Model {
            vertex_buffer,
            index_buffer,
        })
    }

    /// 引数の vertices/indices に追加する。
    fn convert_group(
        group: &Group,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<u32>,
    ) {
        for face in group.faces() {
            let vertex_base = vertices.len();
            let original_vertices: Vec<_> = face.collect();
            for original_vertice in &original_vertices {
                vertices.push(Vertex {
                    position: original_vertice.0.into(),
                    normal: original_vertice
                        .2
                        .unwrap_or(Vec3::new(0.0, 1.0, 0.0))
                        .into(),
                    uv: original_vertice.1.unwrap_or_default().into(),
                });
            }
            for i in 0..(original_vertices.len() - 2) {
                indices.push(vertex_base as u32);
                indices.push((vertex_base + i + 1) as u32);
                indices.push((vertex_base + i + 2) as u32);
            }
        }
    }
}
