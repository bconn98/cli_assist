pub mod cmake;

use clap::{Parser, Subcommand};

/// A CLI for building apps with CMake, Navigating Repositories, and deploying.
#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    // CMake controls
    Cmake(cmake::CmakeVars),
}

pub fn run() {
    let cmds = Args::parse().cmd;

    match cmds {
        Commands::Cmake(v) => {
            cmake::process(v);
        }
    }
}
