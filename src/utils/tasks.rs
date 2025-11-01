use std::sync::Arc;

use anyhow::Result;
use uuid::Uuid;

use crate::{
    api::state::AppState,
    models::{
        a2a::{A2AResponse, Artifact, Message, MessagePart},
        query::QueryParams,
    },
    utils::helpers::{create_artifacts, format_trending_message},
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

    let artifacts = create_artifacts(message.clone());

    call_external_webhook(
        &state.config.external_webhook_url,
        message.clone(),
        artifacts,
    )
    .await?;

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

    let artifacts = create_artifacts(message.clone());

    call_external_webhook(
        &state.config.external_webhook_url,
        message.clone(),
        artifacts,
    )
    .await?;

    tracing::info!("Weekly roundup sent successfully: {}", message);
    Ok(())
}

async fn call_external_webhook(
    webhook_url: &str,
    message: String,
    artifacts: Vec<Artifact>,
) -> Result<()> {
    let client = reqwest::Client::new();

    let request_message = Message {
        kind: "message".to_string(),
        role: "agent".to_string(),
        parts: vec![MessagePart::Text {
            kind: "text".to_string(),
            text: "Proactive notification".to_string(),
        }],
        message_id: Uuid::new_v4().to_string(),
        task_id: Some(Uuid::new_v4().to_string()),
    };

    let payload = serde_json::json!(A2AResponse::success(
        Uuid::new_v4().to_string(),
        Some(Uuid::new_v4().to_string()),
        message,
        artifacts,
        &request_message,
    ));

    let response = client.post(webhook_url).json(&payload).send().await?;

    if !response.status().is_success() {
        anyhow::bail!("Webhook failed: {}", response.status());
    }

    Ok(())
}
