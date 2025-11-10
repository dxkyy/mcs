pub enum ServerType {
    Paper,
    Vanilla,
    Fabric,
    Spigot,
    Forge,
}
#[derive(serde::Deserialize)]
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

#[derive(serde::Deserialize)]
pub struct ModrinthLicense {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
}
#[derive(Debug, serde::Deserialize)]
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

#[derive(Debug, serde::Deserialize)]
pub struct ModrinthFile {
    pub filename: String,
}

pub fn fetch_modrinth_project(project_id: &str) -> Result<ModrinthProject, reqwest::Error> {
    let url = format!("https://api.modrinth.com/v2/project/{}", project_id);
    let response = reqwest::blocking::get(&url)?;
    let project: ModrinthProject = response.json()?;
    Ok(project)
}

pub fn check_compatible_loader(project: &ModrinthProject, server_type: &ServerType) -> bool {
    let compatible_loader = match server_type {
        ServerType::Fabric => "fabric",
        ServerType::Forge => "forge",
        ServerType::Vanilla => "vanilla",
        ServerType::Paper => "paper",
        ServerType::Spigot => "spigot",
    };
    project.loaders.contains(&compatible_loader.to_string())
}

pub fn fetch_versions_for(project_id: &str, loader: &ServerType, game_version: &str) -> Result<Vec<ModrinthProjectVersion>, reqwest::Error> {
    // query parameters: loaders ["fabric", "forge", etc.], game_versions ["1.16.5", etc.] and featured: bool
    let loader_str = match loader {
        ServerType::Fabric => "fabric",
        ServerType::Forge => "forge",
        ServerType::Vanilla => "vanilla",
        ServerType::Paper => "paper",
        ServerType::Spigot => "spigot",
    };
    let url = format!("https://api.modrinth.com/v2/project/{}/version?loaders=[\"{}\"]&game_versions=[\"{}\"]", project_id, loader_str, game_version);
    let response = reqwest::blocking::get(&url)?;
    // print the response text for debugging
    let versions: Vec<serde_json::Value> = response.json()?;
    let mut project_versions = Vec::new();
    for version in versions {
        let project_version: ModrinthProjectVersion = serde_json::from_value(version).unwrap();
        project_versions.push(project_version);
    }
    Ok(project_versions)
}

pub fn download_version_file(project_id: &str, version_id: &str, filename: &str) -> Result<Vec<u8>, reqwest::Error> {
    // https://cdn.modrinth.com/data/AANobbMI/versions/ryOMVRuG/sodium-fabric-0.5.12-beta.2%2Bmc1.20.1.jar
    let url = format!("https://cdn.modrinth.com/data/{}/versions/{}/{}", project_id, version_id, filename);
    let response = reqwest::blocking::get(&url)?;
    let bytes = response.bytes()?;
    Ok(bytes.to_vec())
}

pub fn main() {
    let project_id = "sodium";
    match fetch_modrinth_project(project_id) {
        Ok(project) => {
            println!("Project Title: {}", project.title);
            println!("Project ID: {}", project.id);
            println!("Description: {}", project.description);
            println!("Downloads: {}", project.downloads);
            println!("Compatible with Fabric: {}", check_compatible_loader(&project, &ServerType::Fabric));
        }
        Err(e) => eprintln!("Error fetching project: {}", e),
    }

        // Fetch versions for the project
        match fetch_versions_for(project_id, &ServerType::Fabric, "1.20.1") {
            Ok(versions) => {
                // print full version[0] info for debugging with all fields
                for version in &versions {
                    println!("Version: {} - {} - {}", version.version_number, version.files[0].filename, version.changelog_url.as_ref().unwrap_or(&"No changelog URL".to_string())
                );
                }
                // print all fields for the first version
                if let Some(first_version) = versions.get(0) {
                    println!("Downloading first version file: {}", first_version.files[0].filename);
                    match download_version_file(&first_version.project_id, &first_version.id, &first_version.files[0].filename) {
                        Ok(file_bytes) => {
                            // Save the downloaded file to disk
                            std::fs::write(&first_version.files[0].filename, file_bytes).unwrap();
                            println!("File downloaded and saved as {}", first_version.files[0].filename);
                        }
                        Err(e) => eprintln!("Error downloading file: {}", e),
                    }
                }
            }
            Err(e) => eprintln!("Error fetching versions: {}", e),
        }
    }