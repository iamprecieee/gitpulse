use axum::{Json, extract::State, response::IntoResponse};
use reqwest::StatusCode;
use serde_json::json;
use uuid::Uuid;

use crate::{
    api::state::AppState,
    models::a2a::{A2ARequest, A2AResponse},
    utils::helpers::{extract_user_query, format_trending_message},
};

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200),
    )
)]
pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status": "OK"})))
}

#[utoipa::path(
    post,
    path = "/trending",
    responses(
        (status = 200, body = A2AResponse),
        (status = 500, body = A2AResponse)
    )
)]
pub async fn get_trending(
    State(state): State<AppState>,
    Json(request): Json<A2ARequest>,
) -> impl IntoResponse {
    tracing::info!("Received A2A request: ?{}", request.id);

    let user_text = match extract_user_query(&request) {
        Some(text) => text,
        None => {
            tracing::error!("Failed to extract user query from request");

            return A2AResponse::error(request.id, -32600, "no message text found".to_string())
                .into_response();
        }
    };

    tracing::info!("User query: {}", user_text);

    let params = match state.query_parser.parse(&user_text).await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Failed to parse query with LLM: {}", e);
            return A2AResponse::error(request.id, -32603, format!("Failed to parse query: {}", e))
                .into_response();
        }
    };

    let repos = match state.github_client.search_with_params(&params).await {
        Ok(repos) => repos,
        Err(e) => {
            tracing::error!("GitHub API error: {}", e);

            return A2AResponse::error(
                request.id,
                -32600,
                format!("Failed to fetch trending repos: {}", e).to_string(),
            )
            .into_response();
        }
    };

    let timeframe_label = format!("last {}", params.timeframe);
    let response_text = format_trending_message(&repos, &timeframe_label);
    let response = A2AResponse::success(
        request.id,
        Some(
            request
                .params
                .message
                .task_id
                .unwrap_or_else(|| Uuid::new_v4().to_string()),
        ),
        response_text,
    );

    tracing::info!("Sending successful response with {} repos", repos.len());

    response.into_response()
}
