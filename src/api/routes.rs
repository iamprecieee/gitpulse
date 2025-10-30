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
        (status = -32600, body = A2AResponse),
        (status = -32700, body = A2AResponse)
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

    let params = if let Some(cached_params) = state.cache.get_llm(&user_text) {
        cached_params
    } else {
        match state.query_parser.parse(&user_text).await {
            Ok(param) => {
                tracing::info!("Parsed parameters: {:?}", param);

                state.cache.set(Some(&user_text), &param, None);
                param
            }
            Err(e) => {
                tracing::error!("Failed to parse query with LLM: {}", e);
                return A2AResponse::error(
                    request.id,
                    -32700,
                    "Unable to process your query. Please try rephrasing.".to_string(),
                )
                .into_response();
            }
        }
    };

    let repos = if let Some(cached_repos) = state.cache.get_repo(&params) {
        cached_repos
    } else {
        match state.github_client.search_with_params(&params).await {
            Ok(repos) => {
                state.cache.set(None, &params, Some(repos.clone()));
                repos
            }
            Err(e) => {
                tracing::error!("GitHub API error: {}", e);

                return A2AResponse::error(
                    request.id,
                    -32600,
                    format!("Failed to fetch trending repos: {}", e).to_string(),
                )
                .into_response();
            }
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
