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

pub fn fetch_modrinth_project(project_id: &str) -> Result<ModrinthProject, reqwest::Error> {
    let url = format!("https://api.modrinth.com/v2/project/{}", project_id);
    let response = reqwest::blocking::get(&url)?;
    let project: ModrinthProject = response.json()?;
    Ok(project)
}

pub fn main() {
    let project_id = "sodium";
    match fetch_modrinth_project(project_id) {
        Ok(project) => {
            println!("Project Title: {}", project.title);
            println!("Description: {}", project.description);
            println!("Downloads: {}", project.downloads);
        }
        Err(e) => eprintln!("Error fetching project: {}", e),
    }
}