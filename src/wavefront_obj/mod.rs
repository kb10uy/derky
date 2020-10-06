//! Wavefront OBJ 関係のモジュール。

mod parser;

use crate::AnyResult;
use parser::Parser;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::prelude::*,
    num::NonZeroUsize,
};

use ultraviolet::{Vec2, Vec3};

/// Wavefront OBJ 内で発生するエラーを表す。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Error {
    /// v, vt, vn などで要素数が不足している。
    NotEnoughData { found: usize, expected: usize },

    /// f で不正な頂点定義がある
    InvalidFaceVertex,

    /// f で不正な指定がある
    InvalidIndex,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::NotEnoughData { found, expected } => write!(
                f,
                "Not enough data (found {}, expected {})",
                found, expected
            ),
            Error::InvalidFaceVertex => write!(f, "Invalid face vertex definition"),
            Error::InvalidIndex => write!(f, "Invalid index definition"),
        }
    }
}

impl StdError for Error {}

/// Wavefront OBJ の内容を表す。
#[derive(Debug, Clone)]
pub struct WavefrontObj {
    objects: Box<[Object]>,
    materials: Box<[Material]>,
}

#[allow(dead_code)]
impl WavefrontObj {
    /// このオブジェクトに含まれる全てのグループを返す。
    pub fn objects(&self) -> &[Object] {
        &self.objects
    }

    pub fn materials(&self) -> &[Material] {
        &self.materials
    }

    pub fn from_reader(reader: impl Read) -> AnyResult<WavefrontObj> {
        let mut parser = Parser::new();
        parser.parse_obj(reader)?;
        Ok(parser.into_obj())
    }
}

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
    name: Option<String>,

    /// 割り当てられているマテリアル名
    material_name: Option<String>,

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

/// .mtl ファイルで定義されるマテリアル情報を表す。
#[derive(Debug, Clone)]
pub struct Material;
