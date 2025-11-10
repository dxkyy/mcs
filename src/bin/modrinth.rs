use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum ModrinthError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    
    #[error("JSON parsing failed: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("File operation failed: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("No files found in version")]
    NoFilesFound,
    
    #[error("Invalid project ID: {0}")]
    InvalidProjectId(String),
}

pub type Result<T> = std::result::Result<T, ModrinthError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServerType {
    Paper,
    Vanilla,
    Fabric,
    Spigot,
    Forge,
}

impl ServerType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ServerType::Fabric => "fabric",
            ServerType::Forge => "forge",
            ServerType::Vanilla => "vanilla",
            ServerType::Paper => "paper",
            ServerType::Spigot => "spigot",
        }
    }
}

impl fmt::Display for ServerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// API Response Types
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModrinthProject {
    pub client_side: String,
    pub server_side: String,
    pub game_versions: Vec<String>,
    pub id: String,
    pub slug: String,
    pub project_type: String,
    pub team: String,
    pub organization: Option<String>,
    pub description: String,
    pub title: String,
    pub body: String,
    pub body_url: Option<String>,
    pub published: String,
    pub updated: String,
    pub approved: String,
    pub queued: Option<String>,
    pub status: String,
    pub requested_status: Option<String>,
    pub moderator_message: Option<String>,
    pub license: Option<ModrinthLicense>,
    pub downloads: u64,
    pub followers: u64,
    pub categories: Vec<String>,
    pub additional_categories: Vec<String>,
    pub loaders: Vec<String>,
    pub versions: Vec<String>,
    pub icon_url: Option<String>,
    pub issues_url: Option<String>,
    pub wiki_url: Option<String>,
    pub source_url: Option<String>,
    pub discord_url: Option<String>,
    pub color: Option<u64>,
    pub thread_id: Option<String>,
}

impl ModrinthProject {
    pub fn is_compatible_with(&self, server_type: ServerType) -> bool {
        self.loaders.iter().any(|l| l.eq_ignore_ascii_case(server_type.as_str()))
    }
    
    pub fn compatible_loaders(&self) -> Vec<ServerType> {
        let mut loaders = Vec::new();
        for loader in &[ServerType::Fabric, ServerType::Forge, ServerType::Vanilla, 
                       ServerType::Paper, ServerType::Spigot] {
            if self.is_compatible_with(*loader) {
                loaders.push(*loader);
            }
        }
        loaders
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModrinthLicense {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModrinthProjectVersion {
    pub name: String,
    pub version_number: String,
    pub changelog: Option<String>,
    pub game_versions: Vec<String>,
    pub version_type: String,
    pub loaders: Vec<String>,
    pub featured: bool,
    pub status: Option<String>,
    pub requested_status: Option<String>,
    pub id: String,
    pub project_id: String,
    pub author_id: String,
    pub downloads: u64,
    pub changelog_url: Option<String>,
    pub files: Vec<ModrinthFile>,
}

impl ModrinthProjectVersion {
    pub fn primary_file(&self) -> Option<&ModrinthFile> {
        self.files.first()
    }
    
    pub fn supports_game_version(&self, version: &str) -> bool {
        self.game_versions.contains(&version.to_string())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModrinthFile {
    pub filename: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub primary: Option<bool>,
}

// ============================================================================
// API Client
// ============================================================================

pub struct ModrinthClient {
    client: reqwest::blocking::Client,
    base_url: String,
    cdn_url: String,
}

impl ModrinthClient {
    pub fn new() -> Result<Self> {
        let client = reqwest::blocking::Client::builder()
            .user_agent("github.com/dxkyy/mcs")
            .build()?;
        
        Ok(Self {
            client,
            base_url: "https://api.modrinth.com/v2".to_string(),
            cdn_url: "https://cdn.modrinth.com".to_string(),
        })
    }
    
    pub fn fetch_project(&self, project_id: &str) -> Result<ModrinthProject> {
        if project_id.trim().is_empty() {
            return Err(ModrinthError::InvalidProjectId(project_id.to_string()));
        }
        
        let url = format!("{}/project/{}", self.base_url, project_id);
        let response = self.client.get(&url).send()?;
        
        if !response.status().is_success() {
            return Err(ModrinthError::RequestError(
                response.error_for_status().unwrap_err()
            ));
        }
        
        Ok(response.json()?)
    }
    
    pub fn fetch_versions(
        &self,
        project_id: &str,
        loaders: &[ServerType],
        game_versions: &[&str],
    ) -> Result<Vec<ModrinthProjectVersion>> {
        let mut url = format!("{}/project/{}/version", self.base_url, project_id);
        
        let mut params = Vec::new();
        
        if !loaders.is_empty() {
            let loader_list: Vec<String> = loaders.iter()
                .map(|l| format!("\"{}\"", l.as_str()))
                .collect();
            params.push(format!("loaders=[{}]", loader_list.join(",")));
        }
        
        if !game_versions.is_empty() {
            let version_list: Vec<String> = game_versions.iter()
                .map(|v| format!("\"{}\"", v))
                .collect();
            params.push(format!("game_versions=[{}]", version_list.join(",")));
        }
        
        if !params.is_empty() {
            url.push_str(&format!("?{}", params.join("&")));
        }
        
        let response = self.client.get(&url).send()?;
        
        if !response.status().is_success() {
            return Err(ModrinthError::RequestError(
                response.error_for_status().unwrap_err()
            ));
        }

        Ok(response.json()?)
    }
    
    pub fn download_file(
        &self,
        project_id: &str,
        version_id: &str,
        filename: &str,
    ) -> Result<Vec<u8>> {
        let url = format!(
            "{}/data/{}/versions/{}/{}",
            self.cdn_url, project_id, version_id, filename
        );
        
        let response = self.client.get(&url).send()?;
        
        if !response.status().is_success() {
            return Err(ModrinthError::RequestError(
                response.error_for_status().unwrap_err()
            ));
        }
        
        Ok(response.bytes()?.to_vec())
    }
    
    pub fn download_version(&self, version: &ModrinthProjectVersion) -> Result<Vec<u8>> {
        let file = version.primary_file()
            .ok_or(ModrinthError::NoFilesFound)?;
        
        self.download_file(&version.project_id, &version.id, &file.filename)
    }
    
    pub fn download_and_save(
        &self,
        version: &ModrinthProjectVersion,
        output_path: &std::path::Path,
    ) -> Result<()> {
        let data = self.download_version(version)?;
        std::fs::write(output_path, data)?;
        Ok(())
    }
}

impl Default for ModrinthClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default ModrinthClient")
    }
}


pub fn main() -> Result<()> {
    let client = ModrinthClient::new()?;
    
    // Fetch project information
    let project = client.fetch_project("sodium")?;
    println!("Project Title: {}", project.title);
    println!("Project ID: {}", project.id);
    println!("Description: {}", project.description);
    println!("Downloads: {}", project.downloads);
    println!("Compatible loaders: {:?}", project.compatible_loaders());
    
    // Fetch versions
    let versions = client.fetch_versions(
        &project.id,
        &[ServerType::Fabric],
        &["1.20.1"],
    )?;
    
    println!("\nAvailable versions:");
    for version in &versions {
        if let Some(file) = version.primary_file() {
            println!(
                "  {} - {} - {}",
                version.version_number,
                file.filename,
                version.changelog_url.as_deref().unwrap_or("No changelog")
            );
        }
    }
    
    // Download the first version
    if let Some(version) = versions.first() {
        if let Some(file) = version.primary_file() {
            println!("\nDownloading: {}", file.filename);
            client.download_and_save(version, std::path::Path::new(&file.filename))?;
            println!("File saved successfully!");
        }
    }
    
    Ok(())
}