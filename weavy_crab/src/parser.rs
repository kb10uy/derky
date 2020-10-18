use super::{Error as ObjError, FaceIndexPair, Group, Object, WavefrontObj};
use crate::AnyResult;

use std::{
    error::Error,
    io::{prelude::*, BufReader},
    num::NonZeroUsize,
    str::FromStr,
};

use log::warn;
use ultraviolet::{Vec2, Vec3};

/// .obj ファイル、 .mtl ファイルのパーサー。
#[derive(Debug, Default)]
pub struct Parser<F> {
    include_function: F,
}

impl<F: Fn(&str) -> AnyResult<R>, R: Read> Parser<F> {
    pub fn new(include_function: F) -> Parser<F> {
        Parser { include_function }
    }

    pub fn parse(&self, reader: impl Read) -> AnyResult<WavefrontObj> {
        let mut reader = BufReader::new(reader);

        let mut line_buffer = String::with_capacity(1024);
        let mut obj_buffer = ObjBuffer::default();
        let mut mtl_buffer = MtlBuffer::default();
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

            self.process_obj_line(&mut obj_buffer, &mut mtl_buffer, keyword, &data)?;
        }
        obj_buffer.commit_object();
        mtl_buffer.commit_material();

        Ok(WavefrontObj {
            objects: obj_buffer.complete_objects.into_boxed_slice(),
            materials: mtl_buffer.complete_materials.into_boxed_slice(),
        })
    }

    fn process_obj_line(
        &self,
        obj_buffer: &mut ObjBuffer,
        mtl_buffer: &mut MtlBuffer,
        keyword: &str,
        data: &[&str],
    ) -> AnyResult<()> {
        match keyword {
            "mtllib" => {
                let path = data.get(0).ok_or_else(|| ObjError::PathNotFound)?;
                let include_function = &self.include_function;
                let mtl_reader = include_function(path)?;
                parse_mtl(mtl_buffer, mtl_reader)?;
            }
            "o" => {
                obj_buffer.commit_object();
                obj_buffer.object_buffer.name = data.get(0).map(|&s| s.to_owned());
            }
            "g" => {
                obj_buffer.commit_group();
                obj_buffer.group_buffer.name = data.get(0).map(|&s| s.to_owned());
            }
            "usemtl" => {
                obj_buffer.group_buffer.material_name = data.get(0).map(|&s| s.to_owned());
            }
            "v" => {
                obj_buffer.group_buffer.vertices.push(take_vec3(data)?);
            }
            "vt" => {
                obj_buffer.group_buffer.texture_uvs.push(take_vec2(data)?);
            }
            "vn" => {
                obj_buffer
                    .group_buffer
                    .vertex_normals
                    .push(take_vec3(data)?.normalized());
            }
            "f" => {
                let face = parse_face(obj_buffer, data)?;
                obj_buffer.group_buffer.faces.push(face);
            }
            _ => {
                warn!("Unsupported OBJ keyword: {}", keyword);
            }
        }
        Ok(())
    }
}

/// f 要素をパースする。
fn parse_face(
    obj_buffer: &mut ObjBuffer,
    vertices: impl IntoIterator<Item = impl AsRef<str>>,
) -> AnyResult<Box<[FaceIndexPair]>> {
    let mut index_pairs = vec![];
    let vertices = vertices.into_iter();
    let offsets = [
        obj_buffer.index_offsets.0,
        obj_buffer.index_offsets.1,
        obj_buffer.index_offsets.2,
    ];
    for vertex in vertices {
        let indices = vertex
            .as_ref()
            .split('/')
            .zip(offsets.iter())
            .try_fold::<_, _, Result<_, Box<dyn Error + Send + Sync>>>(
                vec![],
                |mut v, (s, offset)| {
                    if s == "" {
                        v.push(None);
                        return Ok(v);
                    }

                    let parsed = s.parse::<usize>()?;
                    let nzvalue =
                        NonZeroUsize::new(parsed - offset).ok_or(ObjError::InvalidIndex)?;
                    v.push(Some(nzvalue));
                    Ok(v)
                },
            )?;

        match indices.len() {
            1 => {
                index_pairs.push(FaceIndexPair(
                    indices[0].ok_or(ObjError::InvalidIndex)?,
                    None,
                    None,
                ));
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

/// イテレーターを消費して T を生成する。
fn take_single<T>(it: impl IntoIterator<Item = impl AsRef<str>>) -> AnyResult<T>
where
    T: FromStr,
    <T as FromStr>::Err: 'static + Error + Send + Sync,
{
    let mut it = it.into_iter();
    let first = it.next().ok_or_else(|| ObjError::NotEnoughData {
        found: 0,
        expected: 1,
    })?;

    Ok(first.as_ref().parse()?)
}

/// イテレーターを消費して Vec2 を生成する。
fn take_vec2(it: impl IntoIterator<Item = impl AsRef<str>>) -> AnyResult<Vec2> {
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
fn take_vec3(it: impl IntoIterator<Item = impl AsRef<str>>) -> AnyResult<Vec3> {
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
