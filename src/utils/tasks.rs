use std::sync::Arc;

use anyhow::Result;
use uuid::Uuid;

use crate::{
    api::state::AppState,
    models::{a2a::A2AResponse, query::QueryParams},
    utils::helpers::format_trending_message,
};

pub async fn send_daily_digest(state: Arc<AppState>) -> Result<()> {
    let params = QueryParams {
        language: None,
        topics: vec![],
        timeframe: "day".to_string(),
        count: 5,
        min_stars: 30,
    };

    let repos = state.github_client.search_with_params(&params).await?;
    let message = format_trending_message(&repos, "yesterday");

    call_external_webhook(&state.config.external_webhook_url, message.clone()).await?;

    tracing::info!("Daily digest sent successfully: {}", message);
    Ok(())
}

pub async fn send_weekly_roundup(state: Arc<AppState>) -> Result<()> {
    let params = QueryParams {
        language: None,
        topics: vec![],
        timeframe: "week".to_string(),
        count: 10,
        min_stars: 50,
    };

    let repos = state.github_client.search_with_params(&params).await?;
    let message = format_trending_message(&repos, "last week");

    call_external_webhook(&state.config.external_webhook_url, message.clone()).await?;

    tracing::info!("Weekly roundup sent successfully: {}", message);
    Ok(())
}

async fn call_external_webhook(webhook_url: &str, message: String) -> Result<()> {
    let client = reqwest::Client::new();

    let payload = serde_json::json!(A2AResponse::success(
        Uuid::new_v4().to_string(),
        Some(Uuid::new_v4().to_string()),
        message
    ));

    let response = client.post(webhook_url).json(&payload).send().await?;

    if !response.status().is_success() {
        anyhow::bail!("Webhook failed: {}", response.status());
    }

    Ok(())
}
