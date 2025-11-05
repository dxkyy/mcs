use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub version: String,
    pub server_type: ServerType,
    pub memory: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ServerType {
    Paper,
}

impl std::fmt::Display for ServerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerType::Paper => write!(f, "Paper"),
        }
    }
}

impl ServerConfig {
    pub fn new(version: String, server_type: ServerType, memory: String) -> Self {
        Self {
            version,
            server_type,
            memory,
        }
    }

    pub fn load(path: &Path) -> Result<Self> {
        let config_path = path.join("mcs.toml");
        let contents = fs::read_to_string(&config_path)
            .context(format!("Failed to read config file at {:?}", config_path))?;
        let config: ServerConfig = toml::from_str(&contents)
            .context("Failed to parse config file")?;
        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let config_path = path.join("mcs.toml");
        let toml = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        fs::write(&config_path, toml)
            .context(format!("Failed to write config file at {:?}", config_path))?;
        Ok(())
    }
}
