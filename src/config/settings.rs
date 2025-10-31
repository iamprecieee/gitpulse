use std::fs;

use anyhow::{Error, Ok, anyhow};
use dotenvy::dotenv;
use envy::from_env;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub llm_provider: String,
    pub github_search_url: String,
    pub github_access_token: String,
    pub external_webhook_url: String,
    pub llm_api_key: String,
    pub llm_model: String,
    pub system_prompt: Option<String>,
    pub cache_ttl: u64,
    pub host: String,
    pub port: u32,
    pub cors_allowed_origins: String,
}

impl Config {
    pub fn load() -> Result<Self, Error> {
        dotenv().ok();

        let mut config = from_env::<Self>().map_err(|e| anyhow!("Configuration error: {}", e))?;

        let system_prompt = fs::read_to_string("system_prompt.txt")
            .map_err(|e| anyhow!("Failed to load system prompt: {}", e))?;

        config.system_prompt = Some(system_prompt);

        Ok(config)
    }
}
