use std::path::{Path, PathBuf};

use anyhow::{format_err, Result};
use derky::{
    common::{
        model::Model,
        texture::{load_ldr_image, ImageData},
    },
    d3d11::{
        buffer::{IndexBuffer, VertexBuffer},
        context::Device,
        texture::Texture,
    },
    d3d11_vertex,
};
use log::info;
use ultraviolet::{Mat4, Vec2, Vec3};

d3d11_vertex!(ModelVertex : MODEL_VERTEX_LAYOUT {
    position: Vec3 => ("POSITION", 0),
    normal: Vec3 => ("NORMAL", 0),
    uv: Vec2 => ("TEXCOORD", 0),
});

pub fn load_obj(
    device: &Device,
    filename: impl AsRef<Path>,
) -> Result<Model<(VertexBuffer<ModelVertex>, IndexBuffer<u32>), Texture>> {
    let transform = Mat4::from_nonuniform_scale(Vec3::new(1.0, 1.0, -1.0));
    let filename = filename.as_ref();
    let base_path = filename
        .parent()
        .ok_or_else(|| format_err!("Invalid path"))?;

    let model = Model::load_obj(
        filename,
        |faces| {
            let mut vertices = vec![];
            let mut indices = vec![];
            for face in &faces[..] {
                let vertex_base = vertices.len();
                for original_vertice in &face[..] {
                    // 右手系モデルを想定
                    // Blender の出力する .obj は bottom-left が (0, 0) になるらしいので(.obj の仕様？)、
                    // この時点で V を反転する
                    let position = transform * original_vertice.0.into_homogeneous_point();
                    let normal = transform
                        * original_vertice
                            .2
                            .unwrap_or(Vec3::new(0.0, 1.0, 0.0))
                            .into_homogeneous_vector();
                    let uv = {
                        let original = original_vertice.1.unwrap_or_default();
                        Vec2::new(original.x, 1.0 - original.y)
                    };

                    vertices.push(ModelVertex {
                        position: position.into(),
                        normal: normal.into(),
                        uv: uv.into(),
                    });
                }
                for i in 0..(face.len() - 2) {
                    indices.push(vertex_base as u32);
                    indices.push((vertex_base + i + 2) as u32);
                    indices.push((vertex_base + i + 1) as u32);
                }
            }
            info!(
                "Vertex group loaded; {} vertices, {} indices",
                vertices.len(),
                indices.len(),
            );
            let vertex_buffer = VertexBuffer::new(device, &vertices)?;
            let index_buffer = IndexBuffer::new(device, &indices)?;
            Ok((vertex_buffer, index_buffer))
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

                ImageData::new(
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

            let texture = Texture::new(device, &image)?;
            Ok(texture)
        },
    )?;
    Ok(model)
}
