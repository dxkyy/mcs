use anyhow::{Result, anyhow};
use reqwest::blocking::Client;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use crate::config::ServerConfig;

pub fn get_available_versions() -> Result<Vec<String>> {
    let client = Client::builder()
        .user_agent("mcs/1.0.0 (github.com/user/mcs)")
        .build()?;

    let response = client
        .get("https://meta.fabricmc.net/v2/versions/game")
        .send()?;

    if !response.status().is_success() {
        return Err(anyhow!("Failed to fetch Fabric versions from API"));
    }

    let data: Value = response.json()?;
    let versions = data
        .as_array()
        .ok_or_else(|| anyhow!("Failed to parse versions from API"))?
        .iter()
        .filter(|v| v["stable"].as_bool().unwrap_or(false))
        .filter_map(|v| v["version"].as_str().map(String::from))
        .collect();

    Ok(versions)
}

pub fn setup_server(path: &PathBuf, config: &ServerConfig) -> Result<()> {
    println!("\n↓ Downloading Fabric server...");

    let client = Client::builder()
        .user_agent("mcs/1.0.0 (github.com/user/mcs)")
        .build()?;

    println!("■ Fetching Fabric loader version...");
    let loader_response = client
        .get("https://meta.fabricmc.net/v2/versions/loader")
        .send()?;

    if !loader_response.status().is_success() {
        return Err(anyhow!("Failed to fetch Fabric loader versions"));
    }

    let loaders: Value = loader_response.json()?;
    let loader_version = loaders
        .as_array()
        .ok_or_else(|| anyhow!("Failed to parse loader versions"))?
        .first()
        .ok_or_else(|| anyhow!("No loader versions found"))?["version"]
        .as_str()
        .ok_or_else(|| anyhow!("Failed to get loader version"))?;

    println!("■ Using Fabric Loader {}", loader_version);

    let installer_response = client
        .get("https://meta.fabricmc.net/v2/versions/installer")
        .send()?;

    if !installer_response.status().is_success() {
        return Err(anyhow!("Failed to fetch Fabric installer versions"));
    }

    let installers: Value = installer_response.json()?;
    let installer_version = installers
        .as_array()
        .ok_or_else(|| anyhow!("Failed to parse installer versions"))?
        .first()
        .ok_or_else(|| anyhow!("No installer versions found"))?["version"]
        .as_str()
        .ok_or_else(|| anyhow!("Failed to get installer version"))?;

    println!("■ Using Fabric Installer {}", installer_version);

    let download_url = format!(
        "https://meta.fabricmc.net/v2/versions/loader/{}/{}/{}/server/jar",
        config.version, loader_version, installer_version
    );

    println!("■ Downloading Fabric server for Minecraft {}...", config.version);
    let jar_response = client.get(&download_url).send()?;

    if !jar_response.status().is_success() {
        return Err(anyhow!(
            "Version {} may not be available for Fabric. Please check the version number.",
            config.version
        ));
    }

    let jar_bytes = jar_response.bytes()?;

    let server_jar_path = path.join("server.jar");
    fs::write(&server_jar_path, jar_bytes)?;
    fs::create_dir_all(path.join("mods"))?;

    println!("✓ Downloaded server.jar");
    println!("✓ Created mods directory");

    Ok(())
}
