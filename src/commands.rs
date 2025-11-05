use anyhow::{Result, anyhow};
use std::fs;
use std::path::PathBuf;
use crate::config::{ServerConfig, ServerType};
use crate::prompt::prompt_for_config;
use crate::setup::{create_start_scripts, create_eula};
use crate::{paper, vanilla, fabric};

pub fn create_new_server(path: &PathBuf) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }

    let entries: Vec<_> = fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name() != "mcs.toml")
        .collect();

    if !entries.is_empty() {
        return Err(anyhow!(
            "Directory is not empty. Please use an empty directory or run 'mcs configure' to reconfigure."
        ));
    }

    let config = prompt_for_config()?;
    config.save(path)?;

    setup_server(path, &config)?;

    println!("\n✓ Server created successfully!");
    println!("► Location: {}", path.display());
    println!("→ Use start.bat (Windows) or start.sh (Linux/Mac) to start the server");

    Ok(())
}

pub fn reconfigure_server() -> Result<()> {
    let current_dir = std::env::current_dir()?;

    if !current_dir.join("mcs.toml").exists() {
        return Err(anyhow!("No mcs.toml found in current directory. Run 'mcs new <path>' first."));
    }

    let config = prompt_for_config()?;
    config.save(&current_dir)?;

    setup_server(&current_dir, &config)?;

    println!("\n✓ Server reconfigured successfully!");

    Ok(())
}

pub fn apply_config() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let config = ServerConfig::load(&current_dir)?;

    setup_server(&current_dir, &config)?;

    println!("\n✓ Configuration applied successfully!");

    Ok(())
}

fn setup_server(path: &PathBuf, config: &ServerConfig) -> Result<()> {
    match config.server_type {
        ServerType::Paper => {
            paper::setup_server(path, config)?;
            create_start_scripts(path, config)?;
            create_eula(path)?;
        }
        ServerType::Vanilla => {
            vanilla::setup_server(path, config)?;
            create_start_scripts(path, config)?;
            create_eula(path)?;
        }
        ServerType::Fabric => {
            fabric::setup_server(path, config)?;
            create_start_scripts(path, config)?;
            create_eula(path)?;
        }
    }
    Ok(())
}
