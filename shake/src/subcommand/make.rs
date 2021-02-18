use crate::{
    cli::{MakeArguments, UpdateArguments},
    data::{Dependencies, Makefile},
    fxc::Fxc,
    subcommand::update::run_update,
};

use std::{
    collections::HashMap,
    fs::{read_to_string, File},
    path::{Path, PathBuf},
    time::SystemTime,
};

use anyhow::{format_err, Context, Result};
use log::{debug, info};
use relative_path::{RelativePath, RelativePathBuf};
use toml::from_str;

pub fn run_make(args: MakeArguments) -> Result<()> {
    let makefile_path = Path::new(&args.makefile);
    let makefile: Makefile =
        from_str(&read_to_string(makefile_path).context("Failed to read makefile")?)
            .context("Failed to parse makefile")?;

    let depfile_path = Path::new(&args.depfile);
    if !depfile_path.exists() {
        info!("Creating dependency file");
        run_update(UpdateArguments {
            makefile: args.makefile.clone(),
            depfile: args.depfile.clone(),
        })?;
    }
    let depfile: Dependencies =
        from_str(&read_to_string(depfile_path).context("Failed to read dependency file")?)
            .context("Failed to parse dependency file")?;

    let mut updates = HashMap::<RelativePathBuf, SystemTime>::new();
    let macros: Vec<_> = args.macro_definitions.iter().map(AsRef::as_ref).collect();
    let system_now = SystemTime::now();
    for dependency in &depfile.dependencies {
        let mut files = vec![dependency.file.as_str()];
        files.append(&mut dependency.dependencies.iter().map(|d| d.as_str()).collect());

        let latest_update = if updates.is_empty() {
            get_latest_update(makefile.general.input_dir.as_str(), &files)?
        } else {
            system_now
        };
        updates.insert(dependency.file.clone(), latest_update);
    }

    let fxc = Fxc::new(
        &makefile.general.input_dir,
        &makefile.general.output_dir,
        RelativePath::new("fxc"),
    );
    for definition in &makefile.outputs {
        let target_update = {
            let mut path = makefile.general.output_dir.clone();
            path.push(&definition.output);

            if Path::new(path.as_str()).exists() {
                debug!("Reading metadata of {:?}", path);
                let file = File::open(&path.as_str())?;
                let metadata = file.metadata().context("Failed to fetch file metadata")?;
                metadata
                    .modified()
                    .context("Latest update time unavailable")?
            } else {
                SystemTime::UNIX_EPOCH
            }
        };
        let source_update = updates
            .get(&definition.input)
            .ok_or_else(|| format_err!("Invalid dependency file state detected"))?;

        if *source_update > target_update {
            eprintln!("Updating \"{}\"", definition.output);
            fxc.compile(definition, &macros)
                .context(format!("Failed to compile \"{}\"", definition.input))?;
        } else {
            eprintln!("\"{}\" is up to date.", definition.output);
        }
    }

    Ok(())
}

fn get_latest_update(base: impl AsRef<Path>, files: &[impl AsRef<Path>]) -> Result<SystemTime> {
    let base_path = PathBuf::from(base.as_ref());

    let mut latest = SystemTime::UNIX_EPOCH;
    for file in files {
        let mut file_path = base_path.clone();
        file_path.push(file.as_ref());

        debug!("Reading metadata of {:?}", file_path);
        let file = File::open(&file_path)?;
        let metadata = file.metadata().context("Failed to fetch file metadata")?;
        let modified = metadata
            .modified()
            .context("Latest update time unavailable")?;
        if modified > latest {
            latest = modified;
        }
    }

    Ok(latest)
}
