//! .obj ファイル、 .mtl ファイルのパーサーの関数群

use super::{Error as ObjError, FaceIndexPair, Group, Object, WavefrontObj};
use crate::AnyResult;

use std::{
    error::Error,
    io::{prelude::*, BufReader},
    num::NonZeroUsize,
};

use log::warn;
use ultraviolet::{Vec2, Vec3};

#[derive(Debug, Default)]
struct GroupBuffer {
    name: Option<String>,
    vertices: Vec<Vec3>,
    vertex_normals: Vec<Vec3>,
    texture_uvs: Vec<Vec2>,
    faces: Vec<Box<[FaceIndexPair]>>,
}

#[derive(Debug, Default)]
struct ObjectBuffer {
    name: Option<String>,
    groups: Vec<Group>,
}

/// .obj ファイル、 .mtl ファイルのパーサー。
#[derive(Debug, Default)]
pub struct Parser {
    index_offsets: (usize, usize, usize),
    object_buffer: ObjectBuffer,
    group_buffer: GroupBuffer,
    complete_objects: Vec<Object>,
    complete_groups: Vec<Group>,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            ..Default::default()
        }
    }

    pub fn into_obj(self) -> WavefrontObj {
        WavefrontObj {
            objects: self.complete_objects.into_boxed_slice(),
            materials: vec![].into_boxed_slice(),
        }
    }

    pub fn parse_obj(&mut self, reader: impl Read) -> AnyResult<()> {
        let mut reader = BufReader::new(reader);

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

            self.process_obj_line(keyword, &data)?;
        }
        self.commit_object();
        Ok(())
    }

    fn process_obj_line(&mut self, keyword: &str, data: &[&str]) -> AnyResult<()> {
        match keyword {
            "o" => {
                self.commit_object();
                self.object_buffer.name = data.get(0).map(|&s| s.to_owned());
            }
            "g" => {
                self.commit_group();
                self.group_buffer.name = data.get(0).map(|&s| s.to_owned());
            }
            "v" => {
                self.group_buffer.vertices.push(Parser::take_vec3(data)?);
            }
            "vt" => {
                self.group_buffer.texture_uvs.push(Parser::take_vec2(data)?);
            }
            "vn" => {
                self.group_buffer
                    .vertex_normals
                    .push(Parser::take_vec3(data)?.normalized());
            }
            "f" => {
                self.group_buffer.faces.push(self.parse_face(data)?);
            }
            _ => {
                warn!("Unsupported OBJ keyword: {}", keyword);
            }
        }
        Ok(())
    }

    fn commit_object(&mut self) {
        self.commit_group();
        if self.complete_groups.len() > 0 {
            let object = Object {
                name: self.object_buffer.name.clone(),
                groups: self.complete_groups.clone().into_boxed_slice(),
            };
            self.complete_objects.push(object);
            self.complete_groups = vec![];
        }

        self.object_buffer = Default::default();
    }

    fn commit_group(&mut self) {
        self.index_offsets = (
            self.index_offsets.0 + self.group_buffer.vertices.len(),
            self.index_offsets.1 + self.group_buffer.texture_uvs.len(),
            self.index_offsets.2 + self.group_buffer.vertex_normals.len(),
        );
        if self.group_buffer.faces.len() > 0 {
            let group = Group {
                name: self.group_buffer.name.clone(),
                material_name: None,
                vertices: self.group_buffer.vertices.clone().into_boxed_slice(),
                texture_uvs: self.group_buffer.texture_uvs.clone().into_boxed_slice(),
                normals: self.group_buffer.vertex_normals.clone().into_boxed_slice(),
                face_index_pairs: self.group_buffer.faces.clone().into_boxed_slice(),
            };
            self.complete_groups.push(group);
        }
        self.group_buffer = Default::default();
    }

    fn parse_face(
        &self,
        vertices: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> AnyResult<Box<[FaceIndexPair]>> {
        let mut index_pairs = vec![];
        let vertices = vertices.into_iter();
        let offsets = [
            self.index_offsets.0,
            self.index_offsets.1,
            self.index_offsets.2,
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
}
