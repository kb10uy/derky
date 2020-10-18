//! .obj ファイル、 .mtl ファイルのパーサーの関数群

use super::{Error as ObjError, WavefrontObj};

use std::{
    error::Error,
    io::{prelude::*, BufReader},
    num::NonZeroUsize,
    str::FromStr,
};

use log::warn;
use ultraviolet::{Vec2, Vec3};

/// Wavefront OBJ の面の頂点インデックスリストを表す。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FaceIndexPair(NonZeroUsize, Option<NonZeroUsize>, Option<NonZeroUsize>);

/// Wavefront OBJ 内のオブジェクトを表す。
#[derive(Debug, Clone)]
pub struct Object {
    /// 名前
    name: Option<String>,

    /// グループ
    groups: Box<[Group]>,
}

#[allow(dead_code)]
impl Object {
    /// このオブジェクトの名前を返す。
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    ///このオブジェクトのグループを返す。
    pub fn groups(&self) -> &[Group] {
        &self.groups
    }
}

/// Wavefront OBJ 内のグループを表す。
#[derive(Debug, Clone)]
pub struct Group {
    /// 名前
    name: Option<Box<str>>,

    /// 割り当てられているマテリアル名
    material_name: Option<Box<str>>,

    /// 頂点座標
    vertices: Box<[Vec3]>,

    /// 頂点テクスチャ座標
    texture_uvs: Box<[Vec2]>,

    /// 頂点法線(正規)
    normals: Box<[Vec3]>,

    /// 面の頂点ペアのリスト
    face_index_pairs: Box<[Box<[FaceIndexPair]>]>,
}

#[allow(dead_code)]
impl Group {
    /// このグループの名前を返す。
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// このグループの名前を返す。
    pub fn material_name(&self) -> Option<&str> {
        self.material_name.as_deref()
    }

    /// このグループの頂点リストを返す。
    pub fn vertices(&self) -> &[Vec3] {
        &self.vertices
    }

    /// このグループのテクスチャ座標リストを返す。
    pub fn texture_uvs(&self) -> &[Vec2] {
        &self.texture_uvs
    }

    /// このグループの正規化された法線リストを返す。
    pub fn normals(&self) -> &[Vec3] {
        &self.normals
    }

    /// このグループの面のインデックス情報を返す。
    pub fn face_index_pairs(&self) -> &[Box<[FaceIndexPair]>] {
        &self.face_index_pairs
    }

    pub fn faces(&self) -> GroupFaces {
        GroupFaces(self, 0)
    }
}

#[derive(Debug)]
pub struct GroupFaces<'a>(&'a Group, usize);

impl<'a> Iterator for GroupFaces<'a> {
    type Item = FaceVertices<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.1 < self.0.face_index_pairs.len() {
            let result = FaceVertices(self.0, &self.0.face_index_pairs[self.1], 0);
            self.1 += 1;
            Some(result)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct FaceVertices<'a>(&'a Group, &'a [FaceIndexPair], usize);

impl<'a> Iterator for FaceVertices<'a> {
    type Item = (Vec3, Option<Vec2>, Option<Vec3>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.2 < self.1.len() {
            let index_pair = &self.1[self.2];
            let result = (
                self.0.vertices[index_pair.0.get() - 1],
                index_pair.1.map(|nzi| self.0.texture_uvs[nzi.get() - 1]),
                index_pair.2.map(|nzi| self.0.normals[nzi.get() - 1]),
            );
            self.2 += 1;
            Some(result)
        } else {
            None
        }
    }
}

#[derive(Debug, Default)]
struct GroupBuffer {
    name: Option<String>,
    material_name: Option<String>,
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

#[derive(Debug, Default)]
struct ObjBuffer {
    index_offsets: (usize, usize, usize),
    object_buffer: ObjectBuffer,
    group_buffer: GroupBuffer,
    complete_objects: Vec<Object>,
    complete_groups: Vec<Group>,
}

impl ObjBuffer {
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
                material_name: self.group_buffer.material_name.clone(),
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
