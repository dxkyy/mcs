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
        .get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
        .send()?;

    if !response.status().is_success() {
        return Err(anyhow!("Failed to fetch Vanilla versions from Mojang API"));
    }

    let data: Value = response.json()?;
    let versions = data["versions"]
        .as_array()
        .ok_or_else(|| anyhow!("Failed to parse versions from Mojang API"))?
        .iter()
        .filter(|v| v["type"].as_str() == Some("release"))
        .filter_map(|v| v["id"].as_str().map(String::from))
        .collect();

    Ok(versions)
}

pub fn setup_server(path: &PathBuf, config: &ServerConfig) -> Result<()> {
    println!("\n↓ Downloading Vanilla server...");

    let client = Client::builder()
        .user_agent("mcs/1.0.0 (github.com/user/mcs)")
        .build()?;

    let manifest_response = client
        .get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
        .send()?;

    if !manifest_response.status().is_success() {
        return Err(anyhow!("Failed to fetch version manifest from Mojang"));
    }

    let manifest: Value = manifest_response.json()?;

    let versions = manifest["versions"]
        .as_array()
        .ok_or_else(|| anyhow!("Failed to parse versions from manifest"))?;

    let version_data = versions
        .iter()
        .find(|v| v["id"].as_str() == Some(&config.version))
        .ok_or_else(|| anyhow!("Version {} not found", config.version))?;

    let version_url = version_data["url"]
        .as_str()
        .ok_or_else(|| anyhow!("Failed to get version URL"))?;

    println!("■ Found version {}", config.version);

    let version_response = client.get(version_url).send()?;

    if !version_response.status().is_success() {
        return Err(anyhow!("Failed to fetch version details"));
    }

    let version_info: Value = version_response.json()?;

    let server_url = version_info["downloads"]["server"]["url"]
        .as_str()
        .ok_or_else(|| anyhow!("Server download URL not found for this version"))?;

    let jar_response = client.get(server_url).send()?;
    let jar_bytes = jar_response.bytes()?;

    let server_jar_path = path.join("server.jar");
    fs::write(&server_jar_path, jar_bytes)?;

    println!("✓ Downloaded server.jar");

    Ok(())
}
