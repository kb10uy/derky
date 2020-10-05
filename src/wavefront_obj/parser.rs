//! .obj ファイル、 .mtl ファイルのパーサーの関数群

use super::{Error as ObjError, Group, FaceIndexPair};

use std::{error::Error, io::prelude::*, num::NonZeroUsize};

use log::warn;
use ultraviolet::{Vec2, Vec3};

/// .obj ファイルをパースする。
pub fn parse_obj_file(
    mut reader: impl BufRead,
) -> Result<(String, Vec<Group>), Box<dyn Error + Send + Sync>> {
    #[derive(Default)]
    struct GroupBuffer {
        vertices: Vec<Vec3>,
        vertex_normals: Vec<Vec3>,
        texture_uvs: Vec<Vec2>,
        faces: Vec<Box<[FaceIndexPair]>>,
        name: String,
    }

    let mut object_name = String::new();
    let mut groups = vec![];

    let mut group_buffer: GroupBuffer = Default::default();
    let mut named_group = false;
    let mut line_buffer = String::with_capacity(1024);
    loop {
        line_buffer.clear();
        let read_size = reader.read_line(&mut line_buffer)?;
        if read_size == 0 {
            break;
        }

        let trimmed = line_buffer.trim();
        if trimmed == "" || trimmed.starts_with('#') {
            continue;
        }

        let mut elements = line_buffer.trim().split_whitespace();
        let keyword = elements
            .next()
            .expect("Each line should have at least one element");
        let data: Vec<&str> = elements.collect();

        match keyword {
            "o" => {
                object_name = data.get(0).unwrap_or(&"").to_string();
            }
            "g" => {
                if named_group {
                    let group = Group {
                        name: group_buffer.name,
                        vertices: group_buffer.vertices,
                        texture_uvs: group_buffer.texture_uvs,
                        normals: group_buffer.vertex_normals,
                        face_index_pairs: group_buffer.faces,
                    };
                    groups.push(group);
                }

                group_buffer = GroupBuffer {
                    name: data.get(0).unwrap_or(&"").to_string(),
                    ..Default::default()
                };
                named_group = true;
            }
            "v" => {
                group_buffer.vertices.push(take_vec3(data)?);
            }
            "vt" => {
                group_buffer.texture_uvs.push(take_vec2(data)?);
            }
            "vn" => {
                group_buffer
                    .vertex_normals
                    .push(take_vec3(data)?.normalized());
            }
            "f" => {
                group_buffer.faces.push(parse_face(data)?);
            }
            _ => {
                warn!("Unsupported OBJ keyword: {}", keyword);
            }
        }
    }

    // 最後の group_buffer を追加する
    let group = Group {
        name: group_buffer.name,
        vertices: group_buffer.vertices,
        texture_uvs: group_buffer.texture_uvs,
        normals: group_buffer.vertex_normals,
        face_index_pairs: group_buffer.faces,
    };
    groups.push(group);

    Ok((object_name.to_owned(), groups))
}

fn parse_face(
    vertices: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<
    Box<[FaceIndexPair]>,
    Box<dyn Error + Send + Sync>,
> {
    let mut index_pairs = vec![];
    let vertices = vertices.into_iter();
    for vertex in vertices {
        let indices = vertex
            .as_ref()
            .split('/')
            .try_fold::<_, _, Result<_, Box<dyn Error + Send + Sync>>>(vec![], |mut v, s| {
                if s == "" {
                    v.push(None);
                    return Ok(v);
                }

                let parsed = s.parse::<usize>()?;
                let nzvalue = NonZeroUsize::new(parsed).ok_or(ObjError::InvalidIndex)?;
                v.push(Some(nzvalue));
                Ok(v)
            })?;

        match indices.len() {
            1 => {
                index_pairs.push(FaceIndexPair(indices[0].ok_or(ObjError::InvalidIndex)?, None, None));
            }
            3 => {
                index_pairs.push(FaceIndexPair(
                    indices[0].ok_or(ObjError::InvalidIndex)?,
                    indices[1],
                    indices[2],
                ));
            }
            _ => return Err(ObjError::InvalidFaceVertex.into()),
        }
    }

    Ok(index_pairs.into_boxed_slice())
}

/// イテレーターを消費して Vec2 を生成する。
fn take_vec2(
    it: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<Vec2, Box<dyn Error + Send + Sync>> {
    let mut it = it.into_iter();
    let first = it.next().ok_or_else(|| ObjError::NotEnoughData {
        found: 0,
        expected: 2,
    })?;
    let second = it.next().ok_or_else(|| ObjError::NotEnoughData {
        found: 1,
        expected: 2,
    })?;

    Ok(Vec2::new(first.as_ref().parse()?, second.as_ref().parse()?))
}

/// イテレーターを消費して Vec3 を生成する。
fn take_vec3(
    it: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<Vec3, Box<dyn Error + Send + Sync>> {
    let mut it = it.into_iter();
    let first = it.next().ok_or_else(|| ObjError::NotEnoughData {
        found: 0,
        expected: 3,
    })?;
    let second = it.next().ok_or_else(|| ObjError::NotEnoughData {
        found: 1,
        expected: 3,
    })?;
    let third = it.next().ok_or_else(|| ObjError::NotEnoughData {
        found: 2,
        expected: 3,
    })?;

    Ok(Vec3::new(
        first.as_ref().parse()?,
        second.as_ref().parse()?,
        third.as_ref().parse()?,
    ))
}
