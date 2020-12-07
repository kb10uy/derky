mod cli;
mod data;
mod fxc;
mod subcommand {
    mod make;
    mod update;
}

use crate::cli::Arguments;

use clap::Clap;

fn main() {
    pretty_env_logger::init();
    let arguments = Arguments::parse();

    println!("Hello, world!");
}
