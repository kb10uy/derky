use crate::{Error as ObjError, Result, WavefrontObj};

use std::{
    error::Error,
    io::{prelude::*, BufReader},
    num::NonZeroUsize,
    str::FromStr,
};

use log::warn;
use ultraviolet::{Vec2, Vec3};

/// Represents the parser of OBJ/MTL.
#[derive(Debug, Default)]
pub struct Parser<F> {
    include_function: F,
}

impl<F: Fn(&str) -> Result<R>, R: Read> Parser<F> {
    /// Creates an instance of `Parser`.
    /// # Parameters
    /// * `include_function`
    ///     - An resolver closure/function for MTL file
    ///     - When detects `mtllib` command, it tries to resolve the path of
    ///       MTL file. The parser calls this resolver with detected path, so
    ///       you can return any `Read` instance or error.
    pub fn new(include_function: F) -> Parser<F> {
        Parser { include_function }
    }

    /// Parses the OBJ file.
    pub fn parse(&self, reader: impl Read) -> Result<WavefrontObj> {
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

/// Parses a `f` command.
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

    pub(crate) fn into_group(self) -> Group {
        Group {
            name: self.name,
            vertices: self.vertices.into_boxed_slice(),
            texture_uvs: self.texture_uvs.into_boxed_slice(),
            normals: self.normals.into_boxed_slice(),
            face_index_pairs: self.faces.into_boxed_slice(),
        }
    }
}

#[derive(Debug, Default)]
struct ObjectBuffer {
    name: Option<String>,
    complete_groups: Vec<Group>,
    group_buffer: GroupBuffer,
}

impl ObjectBuffer {
    pub(crate) fn process_command(&mut self, key: &str, data: &[&str]) -> Result<()> {
        match key {
            "v" => {
                self.group_buffer.add_vertex(take_vec3(data)?);
            }
            "vt" => {
                self.group_buffer.add_texture_uv(take_vec2(data)?);
            },
            "vn" => {
                self.group_buffer.add_normal(take_vec3(data)?);
            },
            "f" => (),
            _ => unreachable!("Unprocessable command"),
        }

        Ok(())
    }

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
}

#[derive(Debug, Default)]
struct ObjBuffer {
    index_offsets: (usize, usize, usize),
    object_buffer: ObjectBuffer,
    complete_objects: Vec<Object>,
}

impl ObjBuffer {
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
}

#[derive(Debug, Default)]
struct MtlBuffer {
    name: String,
    properties: HashMap<String, MaterialProperty>,
    complete_materials: Vec<Material>,
}

impl MtlBuffer {
    fn commit_material(&mut self) {
        if self.properties.len() > 0 {
            let group = Material {
                name: self.name.clone(),
                properties: self.properties.clone(),
            };
            self.complete_materials.push(group);
        }
        self.properties.clear();
    }
}

/// Parses MTL file.
fn parse_mtl(mtl_buffer: &mut MtlBuffer, reader: impl Read) -> Result<()> {
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

        process_mtl_line(mtl_buffer, keyword, &data)?;
    }

    Ok(())
}

/// Parses a line of MTL file.
fn process_mtl_line(mtl_buffer: &mut MtlBuffer, keyword: &str, data: &[&str]) -> Result<()> {
    match keyword {
        "newmtl" => {
            mtl_buffer.commit_material();
            mtl_buffer.name = data.get(0).unwrap_or(&"").to_string();
        }
        "illum" => {
            let value = take_single(data)?;
            mtl_buffer
                .properties
                .insert("illum".to_owned(), MaterialProperty::Integer(value));
        }
        k if k.starts_with("K") => {
            let value = take_vec3(data)?;
            mtl_buffer
                .properties
                .insert(k.to_owned(), MaterialProperty::Vector(value));
        }
        k if k.starts_with("N") => {
            let value = take_single(data)?;
            mtl_buffer
                .properties
                .insert(k.to_owned(), MaterialProperty::Float(value));
        }
        k if k.starts_with("map_") => {
            let value = data.get(0).unwrap_or(&"").replace("\\\\", "\\");
            let value = PathBuf::from_str(&value).map_err(|_| Error::PathNotFound(value))?;
            mtl_buffer.properties.insert(
                k.to_owned(),
                MaterialProperty::Path(value.into_boxed_path()),
            );
        }
        _ => {
            warn!("Unsupported MTL keyword: {}", keyword);
        }
    }
    Ok(())
}

/// Consumes the iterator and parses the first element.
pub(crate) fn take_single<T>(it: impl IntoIterator<Item = impl AsRef<str>>) -> Result<T>
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
pub(crate) fn take_vec2(it: impl IntoIterator<Item = impl AsRef<str>>) -> Result<Vec2> {
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
pub(crate) fn take_vec3(it: impl IntoIterator<Item = impl AsRef<str>>) -> Result<Vec3> {
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
