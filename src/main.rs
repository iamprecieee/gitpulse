use anyhow::{Error, Ok, Result};
use gitpulse::config::{logging::setup_logging, settings::Config};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _guard = setup_logging();

    let config = Config::load()?;

    tracing::info!(
        "GitHub access token: {}",
        match !config.github_access_token.is_empty() {
            true => "configured",
            false => "not configured",
        }
    );

    tracing::info!(
        "GitHub search url: {}",
        match !config.github_search_url.is_empty() {
            true => "configured",
            false => "not configured",
        }
    );

    tracing::info!(
        "LLM api key: {}",
        match !config.llm_api_key.is_empty() {
            true => "configured",
            false => "not configured",
        }
    );

    tracing::info!(
        "LLM model: {}",
        match !config.llm_model.is_empty() {
            true => "configured",
            false => "not configured",
        }
    );

    tracing::info!(
        "External webhook url: {}",
        match !config.external_webhook_url.is_empty() {
            true => "configured",
            false => "not configured",
        }
    );

    let addr = format!("{}:{}", &config.host, &config.port);

    TcpListener::bind(&addr).await?;

    tracing::info!("Server listening on {}", addr);

    Ok(())
}
