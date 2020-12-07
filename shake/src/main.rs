mod cli;
mod data;
mod fxc;
mod subcommand {
    pub mod make;
    pub mod update;
}

use crate::{
    cli::{Arguments, Subcommands},
    subcommand::{make::run_make, update::run_update},
};

use anyhow::Result;
use clap::Clap;

fn main() -> Result<()> {
    pretty_env_logger::init();
    let arguments = Arguments::parse();

    match arguments.subcommand {
        Subcommands::Make(make_args) => run_make(make_args),
        Subcommands::Update(update_args) => run_update(update_args),
    }
}
