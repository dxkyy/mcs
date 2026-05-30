use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::fs;
use std::path::Path;

const BASE_URL: &str = "https://api.modrinth.com/v2";
const USER_AGENT: &str = "mcs/0.2.0 (github.com/dxkyy/mcs)";

#[derive(Debug, Deserialize)]
pub struct ModrinthProject {
    pub slug: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct ModrinthVersion {
    pub id: String,
    pub name: String,
    pub version_number: String,
    pub loaders: Vec<String>,
    pub game_versions: Vec<String>,
    pub files: Vec<ModrinthFile>,
    pub dependencies: Vec<ModrinthDependency>,
}

#[derive(Debug, Deserialize)]
pub struct ModrinthFile {
    pub url: String,
    pub filename: String,
    pub primary: bool,
}

#[derive(Debug, Deserialize)]
pub struct ModrinthDependency {
    pub project_id: String,
    pub dependency_type: String,
}

fn build_client() -> Result<Client> {
    Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .context("Failed to build HTTP client")
}

pub fn get_project_versions(slug: &str, loader: &str, game_version: &str) -> Result<Vec<ModrinthVersion>> {
    let client = build_client()?;

    let loaders = serde_json::json!([loader]).to_string();
    let game_versions = serde_json::json!([game_version]).to_string();

    let response = client
        .get(format!("{}/project/{}/version", BASE_URL, slug))
        .query(&[
            ("loaders", loaders.as_str()),
            ("game_versions", game_versions.as_str()),
        ])
        .send()
        .context("Failed to fetch versions from Modrinth")?;

    if !response.status().is_success() {
        return Ok(vec![]);
    }

    let versions: Vec<ModrinthVersion> = response
        .json()
        .context("Failed to parse versions response")?;

    Ok(versions)
}

pub fn download_file(url: &str, dest_path: &Path) -> Result<()> {
    let client = build_client()?;

    let response = client
        .get(url)
        .send()
        .context("Failed to download file")?;

    if !response.status().is_success() {
        return Err(anyhow!("Failed to download file: HTTP {}", response.status()));
    }

    let bytes = response.bytes().context("Failed to read download response")?;
    fs::write(dest_path, bytes)
        .context(format!("Failed to write file to {:?}", dest_path))?;

    Ok(())
}

pub fn search_projects(
    query: &str,
    project_type: &str,
    loader: &str,
    game_version: &str,
) -> Result<Vec<ModrinthProject>> {
    let client = build_client()?;

    let facets = serde_json::json!([
        [format!("project_type:{}", project_type)],
        [format!("categories:{}", loader)],
        [format!("versions:{}", game_version)]
    ])
    .to_string();

    let response = client
        .get(format!("{}/search", BASE_URL))
        .query(&[("query", query), ("facets", &facets), ("limit", "5")])
        .send()
        .context("Failed to search Modrinth")?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Modrinth search failed: HTTP {}",
            response.status()
        ));
    }

    #[derive(Deserialize)]
    struct SearchResponse {
        hits: Vec<ModrinthProject>,
    }

    let data: SearchResponse = response
        .json()
        .context("Failed to parse search response")?;

    Ok(data.hits)
}
