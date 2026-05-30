use anyhow::{Result, anyhow};
use std::fs;
use std::path::PathBuf;
use crate::config::{ServerConfig, ServerType};
use crate::prompt::prompt_for_config;
use crate::setup::{create_start_scripts, create_eula};
use crate::{paper, vanilla, fabric, spigot, forge};
use crate::modrinth;
use inquire::Select;

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

pub fn add_content(name: &str) -> Result<()> {
    let current_dir = std::env::current_dir()?;

    if !current_dir.join("mcs.toml").exists() {
        return Err(anyhow!(
            "No mcs.toml found in current directory. Run 'mcs new <path>' first."
        ));
    }

    let config = ServerConfig::load(&current_dir)?;

    let (project_type, loader, dest_subdir) = match config.server_type {
        ServerType::Paper => ("plugin", "paper", "plugins"),
        ServerType::Spigot => ("plugin", "spigot", "plugins"),
        ServerType::Fabric => ("mod", "fabric", "mods"),
        ServerType::Forge => ("mod", "forge", "mods"),
        ServerType::Vanilla => {
            return Err(anyhow!("Vanilla servers don't support mods or plugins."))
        }
    };

    // Extract slug from Modrinth URL or use name as-is
    let slug = if name.contains("modrinth.com") {
        name.trim_end_matches('/')
            .split('/')
            .last()
            .unwrap_or(name)
    } else {
        name
    };

    println!("\n⟳ Fetching versions for '{}'...", slug);

    // Try exact slug lookup; fall back to search if nothing found
    let resolved_slug: String = {
        let versions = modrinth::get_project_versions(slug, loader, &config.version)?;
        if versions.is_empty() {
            // Check if the project exists but is incompatible with this loader/version
            if let Some(project) = modrinth::get_project(slug)? {
                return Err(anyhow!(
                    "'{}' exists on Modrinth but has no versions for {} on Minecraft {}.",
                    project.title, loader, config.version
                ));
            }
            println!("⟳ No exact match, searching Modrinth...");
            let results =
                modrinth::search_projects(name, project_type, loader, &config.version)?;
            if results.is_empty() {
                return Err(anyhow!(
                    "No results found for '{}'. Check the name and try again.",
                    name
                ));
            }
            let display: Vec<String> = results
                .iter()
                .map(|p| format!("{} ({})", p.title, p.slug))
                .collect();
            let choice = Select::new("Select a project:", display.clone()).prompt()?;
            let idx = display.iter().position(|s| s == &choice).unwrap_or(0);
            results[idx].slug.clone()
        } else {
            slug.to_string()
        }
    };

    // Fetch all compatible versions for the resolved slug
    let versions =
        modrinth::get_project_versions(&resolved_slug, loader, &config.version)?;
    if versions.is_empty() {
        return Err(anyhow!(
            "No compatible versions found for '{}' on {} {}.",
            resolved_slug,
            loader,
            config.version
        ));
    }

    // Version picker
    let version_options: Vec<String> = versions
        .iter()
        .map(|v| format!("{} ({})", v.name, v.version_number))
        .collect();
    let choice = Select::new("Select a version:", version_options.clone()).prompt()?;
    let selected_version = versions
        .iter()
        .zip(version_options.iter())
        .find(|(_, opt)| *opt == &choice)
        .map(|(v, _)| v)
        .ok_or_else(|| anyhow!("Failed to match selected version"))?;

    // Warn on required dependencies that aren't already installed
    for dep in &selected_version.dependencies {
        if dep.dependency_type != "required" {
            continue;
        }
        let (dep_title, dep_slug) = modrinth::get_project(&dep.project_id)
            .ok()
            .flatten()
            .map(|p| (p.title, p.slug))
            .unwrap_or_else(|| (dep.project_id.clone(), dep.project_id.clone()));

        let already_installed = fs::read_dir(current_dir.join(dest_subdir))
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .any(|e| e.file_name().to_string_lossy().contains(&dep_slug))
            })
            .unwrap_or(false);

        if !already_installed {
            println!("⚠  This {} requires: {}", project_type, dep_title);
        }
    }

    // Find primary file (fall back to first file if none marked primary)
    let file = selected_version
        .files
        .iter()
        .find(|f| f.primary)
        .or_else(|| selected_version.files.first())
        .ok_or_else(|| anyhow!("No downloadable file found for this version"))?;

    // Create destination dir and download
    let dest_dir = current_dir.join(dest_subdir);
    fs::create_dir_all(&dest_dir)?;

    let dest_path = dest_dir.join(&file.filename);
    println!("\n↓ Downloading {}...", file.filename);
    modrinth::download_file(&file.url, &dest_path)?;

    println!("✓ Installed {}", file.filename);

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
        ServerType::Spigot => {
            spigot::setup_server(path, config)?;
            create_start_scripts(path, config)?;
            create_eula(path)?;
        }
        ServerType::Forge => {
            forge::setup_server(path, config)?;
            create_start_scripts(path, config)?;
            create_eula(path)?;
        }
    }
    Ok(())
}
