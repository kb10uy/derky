//! 描画用モデルに関係するモジュール。

use super::material::Material;
use std::path::{Path, PathBuf};

use anyhow::Result;
use derky::texture::{load_ldr_image, RgbaImageData};
use glium::{
    backend::Facade,
    implement_vertex,
    index::PrimitiveType,
    texture::{RawImage2d, Texture2d},
    IndexBuffer, VertexBuffer,
};
use itertools::Itertools;
use log::info;
use ultraviolet::{Vec3, Vec4};
use weavy_crab::{Material as WavefrontMaterial, WavefrontObj};

/// 頂点シェーダーに渡る頂点情報を表す。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
}
implement_vertex!(Vertex, position, normal, uv);

#[derive(Debug)]
struct ModelGroup {
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u32>,
}

/// VBO/IBO 化したモデルの情報を表す。
#[derive(Debug)]
pub struct Model {
    face_groups: Box<[(VertexBuffer<Vertex>, IndexBuffer<u32>, Option<usize>)]>,
    materials: Box<[Material]>,
}

#[allow(dead_code)]
impl Model {
    pub fn from_obj(
        facade: &impl Facade,
        obj: &WavefrontObj,
        base_path: impl AsRef<Path>,
    ) -> Result<Model> {
        let mut face_groups = vec![];
        let materials = Model::convert_materials(facade, obj.materials(), base_path)?;

        let mut faces: Vec<_> = obj
            .objects()
            .iter()
            .flat_map(|o| o.groups().iter().flat_map(|g| g.faces()))
            .collect();
        faces.sort_by_key(|f| f.1);
        let groups = faces.into_iter().group_by(|f| f.1);
        for (material, faces) in groups.into_iter() {
            let mut vertices = vec![];
            let mut indices = vec![];
            for (face, _) in faces {
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
            let vertex_buffer = VertexBuffer::new(facade, &vertices)?;
            let index_buffer = IndexBuffer::new(facade, PrimitiveType::TrianglesList, &indices)?;
            face_groups.push((vertex_buffer, index_buffer, material));
        }

        Ok(Model {
            face_groups: face_groups.into_boxed_slice(),
            materials,
        })
    }

    pub fn materials(&self) -> &[Material] {
        &self.materials
    }

    /// 全てのグループを巡回する。
    pub fn visit_groups(
        &self,
        mut visitor: impl FnMut(
            &VertexBuffer<Vertex>,
            &IndexBuffer<u32>,
            Option<&Material>,
        ) -> Result<()>,
    ) -> Result<()> {
        for (vb, ib, mi) in &self.face_groups[..] {
            let material = mi.map(|i| self.materials.get(i)).flatten();
            visitor(vb, ib, material)?;
        }

        Ok(())
    }

    fn convert_materials(
        facade: &impl Facade,
        original_materials: &[WavefrontMaterial],
        base_path: impl AsRef<Path>,
    ) -> Result<Box<[Material]>> {
        let mut materials = vec![];

        for original_material in original_materials {
            info!("Loading material {}", original_material.name());
            let image = if let Some(path) = original_material.diffuse_map() {
                let mut filename = PathBuf::from(base_path.as_ref());
                filename.push(path);
                load_ldr_image(filename)?
            } else {
                let color = original_material
                    .diffuse_color()
                    .unwrap_or(Vec3::new(1.0, 1.0, 1.0));
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
            let raw_image = RawImage2d::from_raw_rgba(
                image.into_data().into_vec(),
                (dimensions.0 as u32, dimensions.1 as u32),
            );
            let texture = Texture2d::new(facade, raw_image)?;

            let material = Material::Diffuse {
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                albedo: texture,
            };
            materials.push(material);
        }

        Ok(materials.into_boxed_slice())
    }
}
