use crate::config::ServerConfig;
use anyhow::{Result, anyhow};
use reqwest::blocking::Client;
use serde_json::Value;
use std::cmp::Ordering;
use std::fs;
use std::path::PathBuf;

#[derive(PartialEq, Eq, Debug)]
enum Suffix {
    Release,
    Rc(u64),
    Pre(u64),
}

impl Ord for Suffix {
    fn cmp(&self, other: &Self) -> Ordering {
        let rank = |s: &Suffix| match s {
            Suffix::Release => 2,
            Suffix::Rc(_) => 1,
            Suffix::Pre(_) => 0,
        };
        match (self, other) {
            (Suffix::Rc(a), Suffix::Rc(b)) | (Suffix::Pre(a), Suffix::Pre(b)) => a.cmp(b),
            _ => rank(self).cmp(&rank(other)),
        }
    }
}

impl PartialOrd for Suffix {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(PartialEq, Eq)]
struct Part {
    num: u64,
    suffix: Suffix,
}

impl Ord for Part {
    fn cmp(&self, other: &Self) -> Ordering {
        self.num
            .cmp(&other.num)
            .then_with(|| self.suffix.cmp(&other.suffix))
    }
}

impl PartialOrd for Part {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn parse_part(seg: &str) -> Part {
    let num_len = seg
        .bytes()
        .take_while(|b| b.is_ascii_digit())
        .count();
    let num: u64 = seg[..num_len].parse().unwrap_or(0);
    let rest = seg[num_len..].trim_start_matches('-');

    let suffix = if rest.is_empty() {
        Suffix::Release
    } else if let Some(n) = rest.strip_prefix("rc") {
        Suffix::Rc(n.trim_start_matches('-').parse().unwrap_or(0))
    } else if let Some(n) = rest.strip_prefix("pre") {
        Suffix::Pre(n.trim_start_matches('-').parse().unwrap_or(0))
    } else {
        Suffix::Pre(0)
    };

    Part { num, suffix }
}

fn cmp_versions(a: &str, b: &str) -> Ordering {
    let a_parts: Vec<Part> = a.split('.').map(parse_part).collect();
    let b_parts: Vec<Part> = b.split('.').map(parse_part).collect();

    for (x, y) in a_parts.iter().zip(&b_parts) {
        match x.cmp(y) {
            Ordering::Equal => continue,
            o => return o,
        }
    }
    a_parts.len().cmp(&b_parts.len())
}

pub fn get_available_versions() -> Result<Vec<String>> {
    let client = Client::builder()
        .user_agent("mcs/1.0.0 (github.com/user/mcs)")
        .build()?;

    let response = client
        .get("https://fill.papermc.io/v3/projects/paper")
        .send()?;

    if !response.status().is_success() {
        return Err(anyhow!("Failed to fetch Paper versions from API"));
    }

    let data: Value = response.json()?;
    let mut versions: Vec<String> = data["versions"]
        .as_object()
        .ok_or_else(|| anyhow!("Failed to parse versions from API"))?
        .values()
        .filter_map(|v| v.as_array())
        .flatten()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    versions.sort_by(|a, b| cmp_versions(b, a));

    Ok(versions)
}

pub fn setup_server(path: &PathBuf, config: &ServerConfig) -> Result<()> {
    println!("\n↓ Downloading Paper server...");

    let client = Client::builder()
        .user_agent("mcs/1.0.0 (github.com/user/mcs)")
        .build()?;

    // Check if version exists and get builds
    let builds_url = format!(
        "https://fill.papermc.io/v3/projects/paper/versions/{}/builds",
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
    let builds_array = builds
        .as_array()
        .ok_or_else(|| anyhow!("Failed to parse builds response"))?;

    if builds_array.is_empty() {
        return Err(anyhow!("No builds found for version {}", config.version));
    }

    let latest_build = builds_array
        .last()
        .ok_or_else(|| anyhow!("Failed to get latest build"))?;

    let build_number = latest_build["id"]
        .as_u64()
        .ok_or_else(|| anyhow!("Failed to get build number"))?;

    let download_url = latest_build["downloads"]["server:default"]["url"]
        .as_str()
        .ok_or_else(|| anyhow!("Failed to get download url"))?;

    println!("■ Found build #{}", build_number);

    let jar_response = client.get(download_url).send()?;
    let jar_bytes = jar_response.bytes()?;

    let server_jar_path = path.join("server.jar");
    fs::write(&server_jar_path, jar_bytes)?;
    fs::create_dir_all(path.join("plugins"))?;

    println!("✓ Downloaded server.jar");

    Ok(())
}
