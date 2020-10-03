//! Wavefront OBJ 関係のモジュール。

mod parser;

use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::{prelude::*, BufReader},
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
    name: String,
    groups: Vec<Group>,
    materials: Vec<Material>,
}

impl WavefrontObj {
    /// このオブジェクトの名前を返す。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// このオブジェクトに含まれる全てのグループを返す。
    pub fn groups(&self) -> &[Group] {
        &self.groups
    }

    pub fn from_reader(reader: impl Read) -> Result<WavefrontObj, Box<dyn StdError + Send + Sync>> {
        let (name, groups) = parser::parse_obj_file(BufReader::new(reader))?;

        Ok(WavefrontObj {
            name,
            groups,
            materials: vec![],
        })
    }
}

/// Wavefront OBJ の面の頂点インデックスリストを表す。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FaceIndexPair(NonZeroUsize, Option<NonZeroUsize>, Option<NonZeroUsize>);

/// Wavefront OBJ 内のオブジェクトを表す。
#[derive(Debug, Clone)]
pub struct Group {
    /// 名前
    name: String,

    /// 頂点座標
    vertices: Vec<Vec3>,

    /// 頂点テクスチャ座標
    texture_uvs: Vec<Vec2>,

    /// 頂点法線(正規)
    normals: Vec<Vec3>,

    face_index_pairs: Vec<Box<[FaceIndexPair]>>,
}

impl Group {
    pub fn name(&self) -> &str {
        &self.name
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
    type Item = (Vec3, Option<Vec2>, Option<Vec3>);

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

/// .mtl ファイルで定義されるマテリアル情報を表す。
#[derive(Debug, Clone)]
pub struct Material;
