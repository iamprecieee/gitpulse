use anyhow::{Error, Ok, anyhow};
use dotenvy::dotenv;
use envy::from_env;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub github_search_url: String,
    pub github_access_token: String,
    pub external_webhook_url: String,
    pub llm_api_key: String,
    pub llm_model: String,
    pub cache_ttl: u64,
    pub host: String,
    pub port: u32,
}

impl Config {
    pub fn load() -> Result<Self, Error> {
        dotenv().ok();

        let config = from_env::<Self>().map_err(|e| anyhow!("Configuration error: {}", e))?;

        Ok(config)
    }
}
