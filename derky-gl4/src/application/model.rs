//! 描画用モデルに関係するモジュール。

use super::material::Material;
use std::path::{Path, PathBuf};

use anyhow::{format_err, Result};
use derky::model::Model;
use derky::texture::{load_ldr_image, RgbaImageData};
use glium::{
    backend::Facade,
    implement_vertex,
    index::PrimitiveType,
    texture::{RawImage2d, Texture2d},
    IndexBuffer, VertexBuffer,
};
use log::info;
use ultraviolet::{Vec3, Vec4};

/// 頂点シェーダーに渡る頂点情報を表す。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
}
implement_vertex!(Vertex, position, normal, uv);

#[derive(Debug)]
pub struct ModelGroup {
    pub(crate) vertex_buffer: VertexBuffer<Vertex>,
    pub(crate) index_buffer: IndexBuffer<u32>,
}

pub fn load_obj(
    facade: &impl Facade,
    filename: impl AsRef<Path>,
) -> Result<Model<ModelGroup, Material>> {
    let filename = filename.as_ref();
    let base_path = filename.parent().ok_or_else(|| format_err!("Invalid path"))?;
    Model::load_obj(
        filename,
        |faces| {
            let mut vertices = vec![];
            let mut indices = vec![];
            for face in &faces[..] {
                let vertex_base = vertices.len();
                for original_vertice in &face[..] {
                    vertices.push(Vertex {
                        position: original_vertice.0.into(),
                        normal: original_vertice
                            .2
                            .unwrap_or(Vec3::new(0.0, 1.0, 0.0))
                            .into(),
                        uv: original_vertice.1.unwrap_or_default().into(),
                    });
                }
                for i in 0..(face.len() - 2) {
                    indices.push(vertex_base as u32);
                    indices.push((vertex_base + i + 1) as u32);
                    indices.push((vertex_base + i + 2) as u32);
                }
            }
            let vertex_buffer = VertexBuffer::new(facade, &vertices)?;
            let index_buffer = IndexBuffer::new(facade, PrimitiveType::TrianglesList, &indices)?;
            Ok(ModelGroup {
                vertex_buffer,
                index_buffer,
            })
        },
        |material| {
            info!("Loading material {}", material.name());
            let image = if let Some(path) = material.diffuse_map() {
                let mut filename = PathBuf::from(base_path);
                filename.push(path);
                load_ldr_image(filename)?
            } else {
                let color = material.diffuse_color().unwrap_or(Vec3::new(1.0, 1.0, 1.0));
                info!("Creating dummy image: {:?}", color);

                RgbaImageData::new(
                    &[
                        (color.x * 255.0) as u8,
                        (color.y * 255.0) as u8,
                        (color.z * 255.0) as u8,
                        255,
                    ],
                    1,
                    1,
                )?
            };

            let dimensions = image.dimensions();
            let raw_image = RawImage2d::from_raw_rgba_reversed(
                image.data(),
                (dimensions.0 as u32, dimensions.1 as u32),
            );
            let texture = Texture2d::new(facade, raw_image)?;

            Ok(Material::Diffuse {
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                albedo: texture,
            })
        },
    )
}
