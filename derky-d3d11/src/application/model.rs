use crate::{
    d3d11_vertex,
    rendering::{ComPtr, IndexBuffer, Texture, VertexBuffer},
};

use std::path::{Path, PathBuf};

use anyhow::{format_err, Context, Result};
use derky::{
    model::Model,
    texture::{load_ldr_image, RgbaImageData},
};
use log::info;
use ultraviolet::{Mat3, Vec2, Vec3};
use winapi::{
    shared::{dxgi, dxgiformat, dxgitype, minwindef::HINSTANCE__},
    um::{d3d11, d3dcommon},
    Interface,
};

d3d11_vertex!(ModelVertex : MODEL_VERTEX_LAYOUT {
    position: Vec3 => ("POSITION", 0),
    normal: Vec3 => ("NORMAL", 0),
    uv: Vec2 => ("TEXCOORD", 0),
});

pub fn load_obj(
    device: &ComPtr<d3d11::ID3D11Device>,
    filename: impl AsRef<Path>,
) -> Result<Model<(VertexBuffer<ModelVertex>, IndexBuffer<u32>), Texture>> {
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
                    vertices.push(ModelVertex {
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

            let texture = Texture::new(device, &image)?;
            Ok(texture)
        },
    )?;
    Ok(model)
}
