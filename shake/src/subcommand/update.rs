use crate::{
    cli::UpdateArguments,
    data::{Dependencies, DependencyDefinition, Makefile},
    fxc::Fxc,
};

use std::{
    collections::HashSet,
    fs::{read_to_string, write},
    path::Path,
};

use anyhow::{Context, Result};
use chrono::prelude::*;
use relative_path::RelativePath;
use toml::{from_str, to_string};

pub fn run_update(args: UpdateArguments) -> Result<()> {
    let makefile_path = Path::new(&args.makefile);
    let makefile: Makefile =
        from_str(&read_to_string(makefile_path).context("Failed to read makefile")?)
            .context("Failed to parse makefile")?;

    let fxc = Fxc::new(
        &makefile.general.input_dir,
        &makefile.general.output_dir,
        RelativePath::new("fxc"),
    );

    let mut searched_files = HashSet::new();
    let mut dependencies = vec![];
    for definition in &makefile.outputs {
        let input = definition.input.as_relative_path();
        if searched_files.contains(input) {
            continue;
        }

        let deps = fxc.search_dependencies(definition)?;
        dependencies.push(DependencyDefinition {
            file: input.to_owned(),
            dependencies: deps,
        });
        searched_files.insert(input);
    }

    let depdata = to_string(&Dependencies {
        generated_at: Local::now(),
        dependencies,
    })
    .expect("Failed to export dependency file");
    let depfile = Path::new(&args.depfile);
    write(depfile, depdata).context("Failed to write dependency file")?;

    Ok(())
}
