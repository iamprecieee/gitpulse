use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    pub total_count: u32,
    pub items: Vec<Repository>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Repository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub owner: Owner,
    pub html_url: String,
    pub description: Option<String>,
    pub stargazers_count: u32,
    pub forks_count: u32,
    pub language: Option<String>,
    pub topics: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Owner {
    pub login: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrendingRepo {
    pub name: String,
    pub description: String,
    pub url: String,
    pub language: String,
    pub stars: u32,
}

impl From<Repository> for TrendingRepo {
    fn from(value: Repository) -> Self {
        Self {
            name: value.full_name,
            description: value
                .description
                .unwrap_or_else(|| "No description".to_string()),
            url: value.html_url,
            language: value.language.unwrap_or_else(|| "Unknown".to_string()),
            stars: value.stargazers_count,
        }
    }
}
