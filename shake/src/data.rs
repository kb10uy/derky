//! Contains structs for `shaders.toml` and `shaders.deps`.

use serde::{Deserialize, Serialize};
use chrono::prelude::*;

#[derive(Debug, Deserialize)]
pub struct Makefile {
    general: MakefileGeneral,
    outputs: Vec<MakefileDefinition>,
}

#[derive(Debug, Deserialize)]
pub struct MakefileGeneral {
    input_dir: String,
    output_dir: String,
}

#[derive(Debug, Deserialize)]
pub struct MakefileDefinition {
    input: String,
    output: String,
    profile: String,
    entrypoint: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Dependencies {
    generated_at: DateTime<Local>,
    dependencies: DependencyDefinition,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyDefinition {
    file: String,
    dependencies: Vec<String>,
}
