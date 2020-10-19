use crate::{
    mtl::{Material, MaterialProperty},
    obj::{FaceIndexPair, Group, Object},
    Error, Result, WavefrontObj,
};

use std::{
    collections::HashMap,
    io::{prelude::*, BufReader},
    path::{Path, PathBuf},
    str::FromStr,
};

use log::warn;
use ultraviolet::{Vec2, Vec3};

/// Represents the abstract data of a line in OBJ file.
#[derive(Debug, Clone, PartialEq)]
enum ObjCommand {
    /// `mtllib`
    MaterialLibrary(Box<Path>),

    /// `usemtl`
    UseMaterial(Box<str>),

    /// `o`
    Object(Option<Box<str>>),

    /// `g`
    Group(Option<Box<str>>),

    /// `v`
    Vertex(Vec3),

    /// `vt`
    VertexUv(Vec2),

    /// `vn`
    VertexNormal(Vec3),

    /// `f`
    Face(Box<[FaceIndexPair]>),

    /// Any other unknown keyword
    Unknown(Box<str>, Box<[Box<str>]>),
}

/// Represents the abstract data of a line in MTL file.
#[derive(Debug, Clone, PartialEq)]
enum MtlCommand {
    /// `newmtl`
    NewMaterial(Box<str>),

    /// Integer property
    Integer(Box<str>, u32),

    /// Float property
    Float(Box<str>, f32),

    /// Vector property
    Vector(Box<str>, Vec3),

    /// Path property
    Path(Box<str>, Box<Path>),

    /// Any other unknown keyword
    Unknown(Box<str>, Box<[Box<str>]>),
}

/// Represents the parser of OBJ/MTL.
pub struct Parser<C, R> {
    include_function: Box<dyn FnMut(&Path, &C) -> R>,
}

impl<C, R: Read> Parser<C, R> {
    /// Creates an instance of `Parser`.
    /// # Parameters
    /// * `include_function`
    ///     - An resolver closure/function for MTL file
    ///     - When detects `mtllib` command, it tries to resolve the path of
    ///       MTL file. The parser calls this resolver with detected path and context object,
    ///       so you can return any `Read` instance or error.
    pub fn new(include_function: impl FnMut(&Path, &C) -> R + 'static) -> Parser<C, R> {
        Parser {
            include_function: Box::new(include_function),
        }
    }

    /// Parses the OBJ file.
    pub fn parse(&mut self, reader: impl Read, context: C) -> Result<WavefrontObj> {
        let mut reader = BufReader::new(reader);

        let mut line_buffer = String::with_capacity(1024);
        self.parse_impl(context, move || {
            loop {
                line_buffer.clear();
                let read_size = reader.read_line(&mut line_buffer)?;
                if read_size == 0 {
                    return Ok(None);
                }

                let trimmed = line_buffer.trim();
                if trimmed == "" || trimmed.starts_with('#') {
                    continue;
                }
                break;
            }

            let mut elements = line_buffer.trim().split_whitespace();
            let keyword = elements
                .next()
                .expect("Each line should have at least one element");
            let data: Vec<&str> = elements.collect();
            let command = parse_obj_line(keyword, &data)?;

            Ok(Some(command))
        })
    }

    fn parse_impl<'a>(
        &mut self,
        context: C,
        mut fetch_line: impl FnMut() -> Result<Option<ObjCommand>>,
    ) -> Result<WavefrontObj> {
        let mut materials = Default::default();

        while let Some(command) = fetch_line()? {
            match command {
                ObjCommand::MaterialLibrary(path) => {
                    let mtl_reader = (self.include_function)(&path, &context);
                    materials = parse_mtl(mtl_reader)?;
                }
                _ => {
                    warn!("Unprocessable command: {:?}", command);
                }
            }
        }

        Ok(WavefrontObj {
            materials,
            objects: todo!(),
        })
    }
}

/// Parses a `f` command.
fn parse_face(
    vertices: impl IntoIterator<Item = impl AsRef<str>>,
    vertex_offset: usize,
    uv_offset: usize,
    normal_offset: usize,
) -> Result<Box<[FaceIndexPair]>> {
    let not_enough = |c| Error::NotEnoughData {
        expected: 3,
        found: c,
    };

    let mut index_pairs = vec![];
    for vertex in vertices {
        let indices_str = vertex.as_ref().split('/');
        let mut indices = indices_str.map(|s| {
            if s != "" {
                Some(s.parse::<usize>())
            } else {
                None
            }
        });
        let vertex_index = match indices.next() {
            Some(Some(Ok(v))) => v - 1 - vertex_offset,
            Some(Some(Err(_))) => return Err(Error::ParseError),
            Some(None) => return Err(Error::InvalidFaceVertex),
            None => return Err(not_enough(0)),
        };
        let uv_index = match indices.next() {
            Some(Some(Ok(v))) => Some(v - 1 - uv_offset),
            Some(Some(Err(_))) => return Err(Error::ParseError),
            Some(None) => None,
            None => None,
        };
        let normal_index = match indices.next() {
            Some(Some(Ok(v))) => Some(v - 1 - normal_offset),
            Some(Some(Err(_))) => return Err(Error::ParseError),
            Some(None) => None,
            None => None,
        };
        index_pairs.push(FaceIndexPair(vertex_index, uv_index, normal_index));
    }

    Ok(index_pairs.into_boxed_slice())
}
#[derive(Debug, Default)]
struct GroupBuffer {
    name: Option<Box<str>>,
    vertices: Vec<Vec3>,
    normals: Vec<Vec3>,
    texture_uvs: Vec<Vec2>,
    faces: Vec<Box<[FaceIndexPair]>>,
}

impl GroupBuffer {
    pub(crate) fn new(name: Option<&str>) -> GroupBuffer {
        GroupBuffer {
            name: name.map(|s| s.to_owned().into_boxed_str()),
            ..Default::default()
        }
    }

    pub(crate) fn add_vertex(&mut self, vertex: Vec3) {
        self.vertices.push(vertex);
    }

    pub(crate) fn add_texture_uv(&mut self, texture_uv: Vec2) {
        self.texture_uvs.push(texture_uv);
    }

    pub(crate) fn add_normal(&mut self, normal: Vec3) {
        self.normals.push(normal);
    }

    /*
    pub(crate) fn add_face(
        &mut self,
        index_pairs: impl IntoIterator<Item = FaceIndexPair>,
    ) -> Result<()> {
        let mut face = vec![];
        for index_pair in index_pairs {
            if index_pair.0 >= self.vertices.len() {
                return Err(Error::InvalidIndex);
            }
            match index_pair.1 {
                Some(i) if i >= self.texture_uvs.len() => return Err(Error::InvalidIndex),
                otherwise => otherwise,
            };
            match index_pair.2 {
                Some(i) if i >= self.normals.len() => return Err(Error::InvalidIndex),
                otherwise => otherwise,
            };

            face.push(index_pair);
        }

        self.faces.push(face.into_boxed_slice());
        Ok(())
    }
    */
}

#[derive(Debug, Default)]
struct ObjectBuffer {
    name: Option<String>,
    complete_groups: Vec<Group>,
    group_buffer: GroupBuffer,
}

impl ObjectBuffer {
    /*
    fn commit_group(&mut self, next_name: Option<String>) -> (usize, usize, usize) {
        let offsets = (
            self.group_buffer.vertices.len(),
            self.group_buffer.texture_uvs.len(),
            self.group_buffer.normals.len(),
        );
        if self.group_buffer.faces.len() > 0 {
            let group = self.group_buffer.into_group();
            self.complete_groups.push(group);
            self.group_buffer = Default::default();
            self.group_buffer.name = next_name;
        }

        offsets
    }

    fn into_object(self) -> Object {
        self.commit_group(None);
        Object {
            name: self.name.map(String::into_boxed_str),
            groups: self.complete_groups.into_boxed_slice(),
        }
    }
    */
}

#[derive(Debug, Default)]
struct ObjBuffer {
    index_offsets: (usize, usize, usize),
    object_buffer: ObjectBuffer,
    complete_objects: Vec<Object>,
}

impl ObjBuffer {
    /*
    fn commit_object(&mut self, next_name: Option<String>) {
        let object = self.object_buffer.into_object();
        self.complete_objects.push(object);
        self.object_buffer = Default::default();

        self.commit_group();
        if self.complete_groups.len() > 0 {
            let object = Object {
                name: self.object_buffer.name.clone().map(String::into_boxed_str),
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
                name: self.group_buffer.name.clone().map(String::into_boxed_str),
                vertices: self.group_buffer.vertices.clone().into_boxed_slice(),
                texture_uvs: self.group_buffer.texture_uvs.clone().into_boxed_slice(),
                normals: self.group_buffer.vertex_normals.clone().into_boxed_slice(),
                face_index_pairs: self.group_buffer.faces.clone().into_boxed_slice(),
            };
            self.complete_groups.push(group);
        }
        self.group_buffer = Default::default();
    }
    */
}

/// Parses MTL file.
/// The reader will be wrapped with `BufReader`, so you don't have to
/// do so.
fn parse_mtl(reader: impl Read) -> Result<Box<[Material]>> {
    let mut materials = vec![];
    let mut properties = HashMap::new();
    let mut name = String::new().into_boxed_str();

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

        let command = parse_mtl_line(keyword, &data)?;
        match command {
            MtlCommand::NewMaterial(next_name) => {
                let material = Material { name, properties };
                materials.push(material);

                properties = HashMap::new();
                name = next_name;
            }
            MtlCommand::Vector(n, v) => {
                properties.insert(n.into(), MaterialProperty::Vector(v));
            }
            MtlCommand::Float(n, v) => {
                properties.insert(n.into(), MaterialProperty::Float(v));
            }
            MtlCommand::Integer(n, v) => {
                properties.insert(n.into(), MaterialProperty::Integer(v));
            }
            MtlCommand::Path(n, v) => {
                properties.insert(n.into(), MaterialProperty::Path(v));
            }
            MtlCommand::Unknown(keyword, _) => {
                warn!("Unsupported MTL keyword: {}", keyword);
            }
        }
    }

    Ok(materials.into_boxed_slice())
}

/// Parses a line of OBJ file.
fn parse_obj_line(keyword: &str, data: &[&str]) -> Result<ObjCommand> {
    let value = match keyword {
        "mtllib" => {
            let value = data.get(0).unwrap_or(&"").replace("\\\\", "\\");
            let filename = PathBuf::from_str(&value).map_err(|_| Error::PathNotFound(value))?;
            ObjCommand::MaterialLibrary(filename.into_boxed_path())
        }
        "usemtl" => {
            let material = data.get(0).ok_or(Error::NotEnoughData {
                expected: 1,
                found: 0,
            })?;
            ObjCommand::UseMaterial(material.to_string().into_boxed_str())
        }
        "o" => {
            let name = data.get(0).map(|name| name.to_string().into_boxed_str());
            ObjCommand::Object(name)
        }
        "g" => {
            let name = data.get(0).map(|name| name.to_string().into_boxed_str());
            ObjCommand::Group(name)
        }
        "v" => {
            let value = take_vec3(data)?;
            ObjCommand::Vertex(value)
        }
        "vt" => {
            let value = take_vec2(data)?;
            ObjCommand::VertexUv(value)
        }
        "vn" => {
            let value = take_vec3(data)?;
            ObjCommand::VertexNormal(value)
        }
        "f" => {
            let face = parse_face(data, 0, 0, 0)?;
            ObjCommand::Face(face)
        }
        _ => {
            let owned_data: Vec<_> = data
                .iter()
                .map(|s| s.to_string().into_boxed_str())
                .collect();
            ObjCommand::Unknown(keyword.into(), owned_data.into_boxed_slice())
        }
    };

    Ok(value)
}

/// Parses a line of MTL file.
fn parse_mtl_line(keyword: &str, data: &[&str]) -> Result<MtlCommand> {
    let value = match keyword {
        "newmtl" => {
            let name = data.get(0).unwrap_or(&"").to_string();
            MtlCommand::NewMaterial(name.into_boxed_str())
        }
        "illum" => {
            let value = take_single(data)?;
            MtlCommand::Integer(keyword.into(), value)
        }
        k if k.starts_with("K") => {
            let value = take_vec3(data)?;
            MtlCommand::Vector(keyword.into(), value)
        }
        k if k.starts_with("N") => {
            let value = take_single(data)?;
            MtlCommand::Float(keyword.into(), value)
        }
        k if k.starts_with("map_") => {
            let value = data.get(0).unwrap_or(&"").replace("\\\\", "\\");
            let value = PathBuf::from_str(&value).map_err(|_| Error::PathNotFound(value))?;
            MtlCommand::Path(keyword.into(), value.into_boxed_path())
        }
        _ => {
            let owned_data: Vec<_> = data
                .iter()
                .map(|s| s.to_string().into_boxed_str())
                .collect();
            MtlCommand::Unknown(keyword.into(), owned_data.into_boxed_slice())
        }
    };

    Ok(value)
}

/// Consumes the iterator and parses the first element.
pub(crate) fn take_single<T: FromStr>(it: impl IntoIterator<Item = impl AsRef<str>>) -> Result<T> {
    let mut it = it.into_iter();
    let first = it.next().ok_or_else(|| Error::NotEnoughData {
        found: 0,
        expected: 1,
    })?;

    let value = first.as_ref().parse().map_err(|_| Error::ParseError)?;
    Ok(value)
}

/// Consumes the iterator and parses into `Vec2`.
pub(crate) fn take_vec2(it: impl IntoIterator<Item = impl AsRef<str>>) -> Result<Vec2> {
    let mut it = it.into_iter();
    let first = it
        .next()
        .ok_or_else(|| Error::NotEnoughData {
            found: 0,
            expected: 2,
        })
        .and_then(|s| s.as_ref().parse().map_err(|_| Error::ParseError))?;
    let second = it
        .next()
        .ok_or_else(|| Error::NotEnoughData {
            found: 1,
            expected: 2,
        })
        .and_then(|s| s.as_ref().parse().map_err(|_| Error::ParseError))?;

    Ok(Vec2::new(first, second))
}

/// Consumes the iterator and parses into `Vec3`.
pub(crate) fn take_vec3(it: impl IntoIterator<Item = impl AsRef<str>>) -> Result<Vec3> {
    let mut it = it.into_iter();
    let first = it
        .next()
        .ok_or_else(|| Error::NotEnoughData {
            found: 0,
            expected: 2,
        })
        .and_then(|s| s.as_ref().parse().map_err(|_| Error::ParseError))?;
    let second = it
        .next()
        .ok_or_else(|| Error::NotEnoughData {
            found: 0,
            expected: 2,
        })
        .and_then(|s| s.as_ref().parse().map_err(|_| Error::ParseError))?;
    let third = it
        .next()
        .ok_or_else(|| Error::NotEnoughData {
            found: 0,
            expected: 2,
        })
        .and_then(|s| s.as_ref().parse().map_err(|_| Error::ParseError))?;

    Ok(Vec3::new(first, second, third))
}
