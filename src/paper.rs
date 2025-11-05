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
        .get("https://api.papermc.io/v2/projects/paper")
        .send()?;

    if !response.status().is_success() {
        return Err(anyhow!("Failed to fetch Paper versions from API"));
    }

    let data: Value = response.json()?;
    let versions = data["versions"]
        .as_array()
        .ok_or_else(|| anyhow!("Failed to parse versions from API"))?
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    Ok(versions)
}

pub fn setup_server(path: &PathBuf, config: &ServerConfig) -> Result<()> {
    println!("\n↓ Downloading Paper server...");

    let client = Client::builder()
        .user_agent("mcs/1.0.0 (github.com/user/mcs)")
        .build()?;

    // Check if version exists and get builds
    let builds_url = format!(
        "https://api.papermc.io/v2/projects/paper/versions/{}/builds",
        config.version
    );

    let response = client.get(&builds_url).send()?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Version {} not found or is not available for Paper. Please check the version number.",
            config.version
        ));
    }

    let builds: Value = response.json()?;
    let builds_array = builds["builds"]
        .as_array()
        .ok_or_else(|| anyhow!("Failed to parse builds response"))?;

    if builds_array.is_empty() {
        return Err(anyhow!("No builds found for version {}", config.version));
    }

    let latest_build = builds_array
        .last()
        .ok_or_else(|| anyhow!("Failed to get latest build"))?;

    let build_number = latest_build["build"]
        .as_u64()
        .ok_or_else(|| anyhow!("Failed to get build number"))?;

    let download_name = latest_build["downloads"]["application"]["name"]
        .as_str()
        .ok_or_else(|| anyhow!("Failed to get download name"))?;

    println!("■ Found build #{}", build_number);

    let download_url = format!(
        "https://api.papermc.io/v2/projects/paper/versions/{}/builds/{}/downloads/{}",
        config.version, build_number, download_name
    );

    let jar_response = client.get(&download_url).send()?;
    let jar_bytes = jar_response.bytes()?;

    let server_jar_path = path.join("server.jar");
    fs::write(&server_jar_path, jar_bytes)?;
    fs::create_dir_all(path.join("plugins"))?;


    println!("✓ Downloaded server.jar");

    Ok(())
}
