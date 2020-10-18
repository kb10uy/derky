//! 描画用モデルに関係するモジュール。

use super::material::Material;
use crate::{
    wavefront_obj::{Group, Material as WavefrontMaterial, WavefrontObj},
    AnyResult,
};
use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use glium::{
    backend::Facade,
    implement_vertex,
    index::PrimitiveType,
    texture::{RawImage2d, Texture2d},
    IndexBuffer, VertexBuffer,
};
use image::{io::Reader as ImageReader, Rgba, RgbaImage};
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
struct ModelGroup {
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u32>,
    material_name: String,
}

/// VBO/IBO 化したモデルの情報を表す。
#[derive(Debug)]
pub struct Model {
    groups: Box<[ModelGroup]>,
    materials: HashMap<String, Material>,
}

#[allow(dead_code)]
impl Model {
    pub fn from_obj(
        facade: &impl Facade,
        obj: &WavefrontObj,
        base_path: impl AsRef<Path>,
    ) -> AnyResult<Model> {
        let mut groups = Vec::new();
        let materials = Model::convert_materials(facade, obj.materials(), base_path)?;

        for object in obj.objects() {
            info!("Loading object {:?}", object.name());
            for group in object.groups() {
                info!(
                    "Loading group {:?} , referencing material {:?}",
                    group.name(),
                    group.material_name()
                );
                let mut vertices = vec![];
                let mut indices = vec![];
                Model::convert_group(group, &mut vertices, &mut indices);

                let vertex_buffer = VertexBuffer::new(facade, &vertices)?;
                let index_buffer =
                    IndexBuffer::new(facade, PrimitiveType::TrianglesList, &indices)?;
                groups.push(ModelGroup {
                    vertex_buffer,
                    index_buffer,
                    material_name: group.material_name().unwrap_or("").to_string(),
                })
            }
        }

        Ok(Model {
            groups: groups.into_boxed_slice(),
            materials,
        })
    }

    /// 全てのグループを巡回する。
    pub fn visit_groups(
        &self,
        mut visitor: impl FnMut(
            &VertexBuffer<Vertex>,
            &IndexBuffer<u32>,
            Option<&Material>,
        ) -> AnyResult<()>,
    ) -> AnyResult<()> {
        for group in &self.groups[..] {
            let material = self.materials.get(&group.material_name);
            visitor(&group.vertex_buffer, &group.index_buffer, material)?;
        }

        Ok(())
    }

    /// 引数の vertices/indices に追加する。
    fn convert_group(group: &Group, vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>) {
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

    fn convert_materials(
        facade: &impl Facade,
        original_materials: &[WavefrontMaterial],
        base_path: impl AsRef<Path>,
    ) -> AnyResult<HashMap<String, Material>> {
        let mut materials = HashMap::new();

        for original_material in original_materials {
            info!("Loading material {}", original_material.name());
            let image = if let Some(path) = original_material.diffuse_map() {
                let mut filename = PathBuf::from(base_path.as_ref());
                filename.push(path);

                info!("Loading texture {:?}", filename);
                let file = File::open(filename)?;
                let reader = ImageReader::new(BufReader::new(file)).with_guessed_format()?;
                reader.decode()?.into_rgba()
            } else {
                info!("Creating dummy image");
                let mut image = RgbaImage::new(1, 1);
                image.put_pixel(0, 0, Rgba([255, 255, 255, 255]));
                image
            };

            let dimensions = image.dimensions();
            let raw_image = RawImage2d::from_raw_rgba(image.into_raw(), dimensions);
            let texture = Texture2d::new(facade, raw_image)?;

            let material = Material::Diffuse {
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                albedo: texture,
            };
            materials.insert(original_material.name().to_string(), material);
        }

        Ok(materials)
    }
}
