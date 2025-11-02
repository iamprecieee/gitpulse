use axum::{Json, extract::State, response::IntoResponse};
use reqwest::StatusCode;
use serde_json::json;
use uuid::Uuid;

use crate::{
    api::state::AppState,
    models::a2a::{A2ARequest, A2AResponse},
    utils::helpers::{create_artifacts, extract_user_query, format_trending_message},
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
        (status = 400, body = A2AResponse),
        (status = 500, body = A2AResponse),
    )
)]
pub async fn get_trending(
    State(state): State<AppState>,
    Json(request): Json<A2ARequest>,
) -> impl IntoResponse {
    tracing::info!("Received A2A request: ?{}", request.id);

    if request.jsonrpc != "2.0".to_string() {
        return (
            StatusCode::BAD_REQUEST,
            A2AResponse::error(request.id, -32602, "invalid jsonrpc".to_string()),
        )
            .into_response();
    }

    let user_text = match extract_user_query(&request) {
        Some(text) => text,
        None => {
            tracing::error!("Failed to extract user query from request");

            return (
                StatusCode::BAD_REQUEST,
                A2AResponse::error(request.id, -32600, "no message text found".to_string()),
            )
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
                return (
                    StatusCode::BAD_REQUEST,
                    A2AResponse::error(
                        request.id,
                        -32700,
                        "Unable to process your query. Please try rephrasing.".to_string(),
                    ),
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

                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    A2AResponse::error(
                        request.id,
                        -32600,
                        "Failed to fetch trending repositories. Try again later".to_string(),
                    ),
                )
                    .into_response();
            }
        }
    };

    let timeframe_label = {
        if let Some(created_after) = &params.created_after {
            format!("since {}", created_after)
        } else {
            format!("recent {}", &params.timeframe)
        }
    };
    let response_text = format_trending_message(&repos, &timeframe_label);

    let artifacts = create_artifacts(response_text.clone());

    let response = A2AResponse::success(
        request.id,
        Some(
            request
                .params
                .message
                .task_id
                .clone()
                .unwrap_or_else(|| Uuid::new_v4().to_string()),
        ),
        response_text,
        artifacts,
        &request.params.message,
    );

    tracing::info!("Sending successful response with {} repos", repos.len());

    response.into_response()
}
