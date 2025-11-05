use anyhow::{Result, anyhow};
use reqwest::blocking::Client;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use crate::config::ServerConfig;

pub fn get_available_versions() -> Result<Vec<String>> {
    // Spigot doesn't have a public API for versions, so we'll use the same versions as Vanilla
    // since Spigot supports the same Minecraft versions
    let client = Client::builder()
        .user_agent("mcs/1.0.0 (github.com/user/mcs)")
        .build()?;

    let response = client
        .get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
        .send()?;

    if !response.status().is_success() {
        return Err(anyhow!("Failed to fetch Minecraft versions from Mojang API"));
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
    println!("\n↓ Downloading Spigot server...");

    let client = Client::builder()
        .user_agent("mcs/1.0.0 (github.com/user/mcs)")
        .build()?;

    // Get BuildTools
    println!("■ Downloading Spigot BuildTools...");
    let buildtools_url = "https://hub.spigotmc.org/jenkins/job/BuildTools/lastSuccessfulBuild/artifact/target/BuildTools.jar";

    let buildtools_response = client.get(buildtools_url).send()?;

    if !buildtools_response.status().is_success() {
        return Err(anyhow!("Failed to download Spigot BuildTools"));
    }

    let buildtools_bytes = buildtools_response.bytes()?;
    let buildtools_path = path.join("BuildTools.jar");
    fs::write(&buildtools_path, buildtools_bytes)?;

    println!("✓ Downloaded BuildTools.jar");
    println!("\n⚠ Note: Spigot requires building from source.");
    println!("■ This may take several minutes...");
    println!("■ Building Spigot {} (this happens once per version)...", config.version);

    // Run BuildTools to build Spigot
    let output = std::process::Command::new("java")
        .arg("-jar")
        .arg("BuildTools.jar")
        .arg("--rev")
        .arg(&config.version)
        .current_dir(path)
        .output();

    match output {
        Ok(result) => {
            if !result.status.success() {
                let stderr = String::from_utf8_lossy(&result.stderr);
                return Err(anyhow!("BuildTools failed: {}", stderr));
            }

            // Find the generated spigot jar
            let entries = fs::read_dir(path)?;
            let mut found_jar = false;

            for entry in entries {
                let entry = entry?;
                let filename = entry.file_name();
                let filename_str = filename.to_string_lossy();

                if filename_str.starts_with("spigot-") && filename_str.ends_with(".jar") {
                    fs::rename(entry.path(), path.join("server.jar"))?;
                    found_jar = true;
                    break;
                }
            }

            if !found_jar {
                return Err(anyhow!("Failed to find generated Spigot jar file"));
            }

            let _ = fs::remove_file(&buildtools_path);

            println!("✓ Built Spigot server successfully");

            fs::create_dir_all(path.join("plugins"))?;
            println!("✓ Created plugins directory");
        }
        Err(e) => {
            return Err(anyhow!("Failed to run BuildTools. Make sure Java is installed and in PATH: {}", e));
        }
    }

    Ok(())
}
