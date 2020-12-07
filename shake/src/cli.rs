use clap::Clap;

/// Represents CLI arguments.
#[derive(Clap)]
#[clap(author, version)]
pub struct Arguments {
    /// Subcommand
    #[clap(subcommand)]
    pub subcommand: Subcommands,
}

/// Represents available subcommands.
#[derive(Clap)]
pub enum Subcommands {
    /// Makes all shader definitions.
    Make(MakeArguments),

    /// Updates shader dependency information.
    Update(UpdateArguments),
}

#[derive(Clap)]
pub struct MakeArguments {
    /// Specifies the shader makefile path.
    #[clap(short = 'c', long, default_value = "shaders.toml")]
    pub makefile: String,

    /// Specifies the shader depfile path.
    #[clap(short, long, default_value = "shaders.deps")]
    pub depfile: String,
}

#[derive(Clap)]
pub struct UpdateArguments {
    /// Specifies the shader makefile path.
    #[clap(short = 'c', long, default_value = "shaders.toml")]
    pub makefile: String,

    /// Specifies the shader depfile path.
    #[clap(short, long, default_value = "shaders.deps")]
    pub depfile: String,
}
