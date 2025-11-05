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
        .get("https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json")
        .send()?;

    if !response.status().is_success() {
        return Err(anyhow!("Failed to fetch Forge versions from API"));
    }

    let data: Value = response.json()?;
    let promos = data["promos"]
        .as_object()
        .ok_or_else(|| anyhow!("Failed to parse Forge promotions"))?;

    let mut versions: Vec<String> = promos
        .keys()
        .filter_map(|key| {
            if key.ends_with("-recommended") || key.ends_with("-latest") {
                Some(key.trim_end_matches("-recommended").trim_end_matches("-latest").to_string())
            } else {
                None
            }
        })
        .collect();

    versions.sort();
    versions.dedup();
    versions.reverse();

    if versions.is_empty() {
        return Err(anyhow!("No Forge versions found"));
    }

    Ok(versions)
}

pub fn setup_server(path: &PathBuf, config: &ServerConfig) -> Result<()> {
    println!("\n↓ Downloading Forge server...");

    let client = Client::builder()
        .user_agent("mcs/1.0.0 (github.com/user/mcs)")
        .build()?;

    println!("■ Fetching Forge build information for {}...", config.version);
    let promos_response = client
        .get("https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json")
        .send()?;

    if !promos_response.status().is_success() {
        return Err(anyhow!("Failed to fetch Forge promotions"));
    }

    let promos: Value = promos_response.json()?;
    let promos_obj = promos["promos"]
        .as_object()
        .ok_or_else(|| anyhow!("Failed to parse Forge promotions"))?;

    let recommended_key = format!("{}-recommended", config.version);
    let latest_key = format!("{}-latest", config.version);

    let forge_build = promos_obj
        .get(&recommended_key)
        .or_else(|| promos_obj.get(&latest_key))
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("No Forge build found for Minecraft version {}", config.version))?;

    let forge_version = format!("{}-{}", config.version, forge_build);
    println!("■ Using Forge {}", forge_version);

    let installer_url = format!(
        "https://maven.minecraftforge.net/net/minecraftforge/forge/{}/forge-{}-installer.jar",
        forge_version, forge_version
    );

    println!("■ Downloading Forge installer...");
    let installer_response = client.get(&installer_url).send()?;

    if !installer_response.status().is_success() {
        return Err(anyhow!(
            "Failed to download Forge installer for version {}. The version may not be available.",
            forge_version
        ));
    }

    let installer_bytes = installer_response.bytes()?;
    let installer_path = path.join("forge-installer.jar");
    fs::write(&installer_path, installer_bytes)?;

    println!("✓ Downloaded Forge installer");
    println!("\n■ Installing Forge server (this may take a moment)...");

    let output = std::process::Command::new("java")
        .arg("-jar")
        .arg("forge-installer.jar")
        .arg("--installServer")
        .current_dir(path)
        .output();

    match output {
        Ok(result) => {
            if !result.status.success() {
                let stderr = String::from_utf8_lossy(&result.stderr);
                return Err(anyhow!("Forge installer failed: {}", stderr));
            }

            let entries = fs::read_dir(path)?;
            let mut found_jar = false;

            for entry in entries {
                let entry = entry?;
                let filename = entry.file_name();
                let filename_str = filename.to_string_lossy();

                if filename_str.contains("forge") &&
                   filename_str.ends_with(".jar") &&
                   !filename_str.contains("installer") &&
                   !filename_str.contains("universal") {
                    fs::copy(entry.path(), path.join("server.jar"))?;
                    found_jar = true;
                    break;
                }
            }

            if !found_jar {
                if path.join("run.sh").exists() || path.join("run.bat").exists() {
                    println!("⚠ Note: This Forge version uses run.sh/run.bat for starting.");
                    println!("  The generated start scripts will work, but you can also use the Forge scripts.");
                } else {
                    return Err(anyhow!("Failed to find Forge server jar after installation"));
                }
            }

            let _ = fs::remove_file(&installer_path);

            println!("✓ Installed Forge server successfully");

            fs::create_dir_all(path.join("mods"))?;
            println!("✓ Created mods directory");
        }
        Err(e) => {
            return Err(anyhow!("Failed to run Forge installer. Make sure Java is installed and in PATH: {}", e));
        }
    }

    Ok(())
}
