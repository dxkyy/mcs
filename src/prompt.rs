use anyhow::Result;
use inquire::{Select, Text};
use crate::config::{ServerConfig, ServerType};
use crate::{paper, vanilla};

pub fn prompt_for_config() -> Result<ServerConfig> {
    println!("▶ Minecraft Server Configuration\n");

    let server_types = vec!["Paper", "Vanilla"];
    let server_type_str = Select::new("Server type:", server_types)
        .prompt()?;

    let server_type = match server_type_str {
        "Paper" => ServerType::Paper,
        "Vanilla" => ServerType::Vanilla,
        _ => unreachable!(),
    };

    let version = match server_type {
        ServerType::Paper => {
            println!("\n⟳ Fetching available Paper versions...");
            let versions = paper::get_available_versions()?;

            let mut versions = versions;
            versions.reverse();

            let default_index = 0;

            Select::new("Minecraft version:", versions)
                .with_starting_cursor(default_index)
                .with_help_message("Use arrow keys or type to search")
                .prompt()?
        }
        ServerType::Vanilla => {
            println!("\n⟳ Fetching available Vanilla versions...");
            let versions = vanilla::get_available_versions()?;

            let default_index = 0;

            Select::new("Minecraft version:", versions)
                .with_starting_cursor(default_index)
                .with_help_message("Use arrow keys or type to search")
                .prompt()?
        }
    };

    let memory = Text::new("Memory allocation:")
        .with_default("2G")
        .with_help_message("e.g., 2G, 4G, 8G")
        .prompt()?;

    Ok(ServerConfig::new(version, server_type, memory))
}
