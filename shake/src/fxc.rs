//! Contains operations for `fxc.exe`.

use crate::data::MakefileDefinition;

use std::{
    env::current_dir,
    fs::create_dir_all,
    io::prelude::*,
    path::Path,
    process::ExitStatus,
    process::{Command, Stdio},
};

use anyhow::{Context, Result};
use log::{debug, info};
use once_cell::sync::Lazy;
use regex::Regex;
use relative_path::{RelativePath, RelativePathBuf};

static RE_FXC_RESOLVE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"Resolved to \[(.+)\]"#).expect("Invalid regex"));

/// An abstraction structure for `fxc.exe`.
pub struct Fxc {
    input_directory: RelativePathBuf,
    output_directory: RelativePathBuf,
    fxc_path: RelativePathBuf,
}

impl Fxc {
    /// Creates a new `Fxc` abstraction with directories and compiler path.
    pub fn new(
        input_dir: impl AsRef<RelativePath>,
        output_dir: impl AsRef<RelativePath>,
        fxc: impl AsRef<RelativePath>,
    ) -> Fxc {
        Fxc {
            input_directory: input_dir.as_ref().to_owned(),
            output_directory: output_dir.as_ref().to_owned(),
            fxc_path: fxc.as_ref().to_owned(),
        }
    }

    pub fn compile(&self, definition: &MakefileDefinition) -> Result<ExitStatus> {
        let mut input_path = self.input_directory.clone();
        input_path.push(&definition.input);
        let mut output_path = self.output_directory.clone();
        output_path.push(&definition.output);
        create_dir_all(output_path.parent().map(|p| p.as_str()).unwrap_or(""))?;

        info!(
            "Compiling {} (profile: {}, entrypoint: {})",
            definition.input, definition.profile, definition.entrypoint
        );
        let args = [
            // Don't output the logo, and output include information
            "/nologo",
            // Target profile,
            "/T",
            &definition.profile,
            // Entrypoint,
            "/E",
            &definition.entrypoint,
            // No output file
            // TODO: Support for non-Windows environment
            "/Fo",
            output_path.as_str(),
            // Input file
            input_path.as_str(),
        ];

        debug!("Executing fxc with {:?}", &args);
        let mut command = Command::new(&self.fxc_path.as_str())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .args(&args)
            .spawn()
            .context("Failed to spawn fxc")?;

        let status = command.wait().context("Error occurred in fxc.exe")?;

        Ok(status)
    }

    /// Executes `fxc.exe` and search dependencies for given definition.
    pub fn search_dependencies(
        &self,
        definition: &MakefileDefinition,
    ) -> Result<Vec<RelativePathBuf>> {
        let cwd = current_dir().context("Failed to fetch current directory")?;
        let mut absolute_input_directory = cwd.clone();
        absolute_input_directory.push(self.input_directory.as_str());
        let mut input_path = self.input_directory.clone();
        input_path.push(&definition.input);

        info!("Searching dependencies for {}", input_path);
        let args = [
            // Don't output the logo, and output include information
            "/nologo",
            "/Vi",
            // Target profile,
            "/T",
            &definition.profile,
            // Entrypoint,
            "/E",
            &definition.entrypoint,
            // No output file
            // TODO: Support for non-Windows environment
            "/Fo",
            "NUL",
            // Input file
            input_path.as_str(),
        ];

        debug!("Executing fxc with {:?}", &args);
        let command = Command::new(&self.fxc_path.as_str())
            .stdout(Stdio::piped())
            .args(&args)
            .spawn()
            .context("Failed to spawn fxc")?;

        let mut fxc_output = String::with_capacity(1024);
        let mut stdout = command.stdout.context("Failed to open fxc stdout")?;
        stdout
            .read_to_string(&mut fxc_output)
            .context("Failed to read fxc output")?;

        let mut dependencies = vec![];

        let dependency_captures = RE_FXC_RESOLVE.captures_iter(&fxc_output);
        for capture in dependency_captures {
            let abs_path = Path::new(capture.get(1).expect("Capture group should exist").as_str());
            let rel_path = abs_path
                .strip_prefix(&absolute_input_directory)
                .context("Dependency points outside input diectory")?;
            let rel_path = RelativePathBuf::from_path(&rel_path)?;
            info!("Found dependency: {}", rel_path.as_str());

            dependencies.push(rel_path);
        }

        Ok(dependencies)
    }
}
