mod config;
mod commands;
mod paper;
mod vanilla;
mod fabric;
mod spigot;
mod forge;
mod prompt;
mod setup;
mod modrinth;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "mcs")]
#[command(about = "A CLI for creating Minecraft servers", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    New {
        path: PathBuf,
    },
    Apply,
    Configure,
    Add {
        name: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { path } => {
            commands::create_new_server(&path)?;
        }
        Commands::Apply => {
            commands::apply_config()?;
        }
        Commands::Configure => {
            commands::reconfigure_server()?;
        }
        Commands::Add { name } => {
            commands::add_content(&name)?;
        }
    }

    Ok(())
}
