use anyhow::{Context, Error, Result};
use gitpulse::{
    api::{build_router, state::AppState},
    config::{logging::setup_logging, settings::Config},
    services::{ai::QueryParser, github::GitHubClient},
};
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

    let github_client =
        GitHubClient::new(Some(&config.github_access_token), &config.github_search_url)?;

    tracing::info!("GitHub client initialized");

    let system_prompt = config
        .system_prompt
        .clone()
        .context("System prompt is required but not configured")?;

    let query_parser = QueryParser::new(
        &config.llm_api_key,
        &config.llm_model,
        system_prompt.as_str(),
    )
    .await
    .context("Failed to initialize query parser")?;

    tracing::info!("Query parser initialized");

    let addr = format!("{}:{}", &config.host, &config.port);

    let state = AppState {
        github_client,
        config,
        query_parser,
    };

    let app = build_router(state);

    let listener = TcpListener::bind(&addr).await?;

    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
