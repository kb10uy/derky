//! Parses the Wavefront OBJ format.

mod mtl;
mod obj;
mod parser;

pub use mtl::{Material, MaterialProperty};
pub use obj::{Group, Object, FaceVertexPair};
pub use parser::Parser;

use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IoError,
    result::Result as StdResult,
};

/// Results for Wavefront OBJ/MTL parsing.
pub type Result<T> = StdResult<T, Error>;

/// Represents an error in parsing OBJ/MTL.
#[derive(Debug)]
pub enum Error {
    /// Not enough value defined in `v`, `vt`, `vn`, etc.
    NotEnoughData { found: usize, expected: usize },

    /// Invalid `f` definition detected (referencing undefined vertices).
    InvalidFaceVertex,

    /// Invalid `f` index detected (zero or negative index).
    InvalidIndex,

    /// Specified filename was not found.
    PathNotFound(String),

    /// IO error.
    IoError(IoError),

    /// Parsing error.
    ParseError,
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
            Error::PathNotFound(path) => write!(f, "Path not found: \"{}\"", path),
            Error::IoError(err) => err.fmt(f),
            Error::ParseError => write!(f, "Failed to parse a value"),
        }
    }
}

impl StdError for Error {}

impl From<IoError> for Error {
    fn from(err: IoError) -> Self {
        Error::IoError(err)
    }
}

/// Represents the content of OBJ file and corresponding MTL file.
#[derive(Debug, Clone)]
pub struct WavefrontObj {
    objects: Box<[Object]>,
    materials: Box<[Material]>,
}

impl WavefrontObj {
    /// Object definitions which this OBJ have.
    pub fn objects(&self) -> &[Object] {
        &self.objects
    }

    /// Materials which this OBJ have.
    pub fn materials(&self) -> &[Material] {
        &self.materials
    }

    /// Splits into separate data, objects and materials.
    pub fn split(self) -> (Box<[Object]>, Box<[Material]>) {
        (self.objects, self.materials)
    }
}
