//! Contains structs for `shaders.toml` and `shaders.deps`.

use chrono::prelude::*;
use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};

/// Represents the structure of `shaders.toml`.
#[derive(Debug, Deserialize)]
pub struct Makefile {
    /// `[general]` table
    pub general: MakefileGeneral,

    /// `[[outputs]]` array
    pub outputs: Vec<MakefileDefinition>,
}

/// Represents general information of `shaders.toml`.
#[derive(Debug, Deserialize)]
pub struct MakefileGeneral {
    /// Input directory of shaders
    pub input_dir: RelativePathBuf,

    /// Output directory of shaders
    pub output_dir: RelativePathBuf,
}

/// Represents a definition of `shaders.toml`.
#[derive(Debug, Deserialize)]
pub struct MakefileDefinition {
    /// Input filename
    pub input: RelativePathBuf,

    /// Output filename
    pub output: RelativePathBuf,

    /// Target profile
    pub profile: String,

    /// Entrypoint of the shader
    pub entrypoint: String,
}

/// Represents the structure of `shaders.deps`.
#[derive(Debug, Serialize, Deserialize)]
pub struct Dependencies {
    /// `DateTime` at which this dependencies created
    pub generated_at: DateTime<Local>,

    /// Dependencies
    pub dependencies: Vec<DependencyDefinition>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyDefinition {
    /// Filename
    pub file: RelativePathBuf,

    /// Dependency filename
    pub dependencies: Vec<RelativePathBuf>,
}
