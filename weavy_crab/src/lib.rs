//! Parses the Wavefront OBJ format.

mod mtl;
mod obj;
mod parser;

pub use parser::Parser;
use std::{
    collections::HashMap,
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    num::NonZeroUsize,
    path::Path,
};

use ultraviolet::{Vec2, Vec3};

/// Represents an error in parsing OBJ/MTL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Error {
    /// Not enough value defined in `v`, `vt`, `vn`, etc.
    NotEnoughData { found: usize, expected: usize },

    /// Invalid `f` definition detected (referencing undefined vertices).
    InvalidFaceVertex,

    /// Invalid `f` index detected (zero or negative index).
    InvalidIndex,

    /// Specified filename was not foud.
    PathNotFound,
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
            Error::PathNotFound => write!(f, "Path not found"),
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
}
