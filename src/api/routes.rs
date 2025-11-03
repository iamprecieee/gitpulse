use axum::{
    Json,
    body::Bytes,
    extract::State,
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};
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
    ),
    tag = "health",
)]
pub async fn health_check() -> Response {
    Json(json!({"status": "OK"})).into_response()
}

#[utoipa::path(
    post,
    path = "/trending",
    request_body = A2ARequest,
    tag = "A2A",
)]
pub async fn get_trending(State(state): State<AppState>, body: Bytes) -> Response {
    if body.is_empty() {
        tracing::warn!("Received empty request body");
        return Json(A2AResponse::error(
            -32600,
            "Empty request received".to_string(),
        ))
        .into_response();
    }

    let parsed_json_value: Value = match serde_json::from_slice(&body) {
        Ok(val) => val,
        Err(e) => {
            tracing::error!("JSON parse error: {}", e);

            return Json(A2AResponse::error(
                -32700,
                "Parse error: Invalid JSON".to_string(),
            ))
            .into_response();
        }
    };

    if parsed_json_value
        .as_object()
        .map_or(false, |obj| obj.is_empty())
    {
        tracing::info!("Received empty JSON object");

        return Json(A2AResponse::error(
            -32600,
            "Empty JSON object received".to_string(),
        ))
        .into_response();
    }

    let request: A2ARequest = match serde_json::from_value(parsed_json_value.clone()) {
        Ok(req) => req,
        Err(e) => {
            tracing::error!("A2ARequest deserialization error: {}", e);

            return Json(A2AResponse::error(
                -32602,
                "Required fields may be missing or have wrong types".to_string(),
            ))
            .into_response();
        }
    };

    get_trending_inner(state, request).await
}

async fn get_trending_inner(state: AppState, request: A2ARequest) -> Response {
    tracing::info!("Received A2A request: ?{}", request.id);

    if request.jsonrpc != "2.0".to_string() {
        return Json(A2AResponse::error(
            -32602,
            "Invalid params: jsonrpc must be '2.0'".to_string(),
        ))
        .into_response();
    }

    if request.method != "message/send" {
        return Json(A2AResponse::error(-32601, "Method not found".to_string())).into_response();
    }

    let user_text = match extract_user_query(&request) {
        Some(text) => text,
        None => {
            tracing::error!("Failed to extract user query from request");

            return Json(A2AResponse::error(
                -32602,
                "no message text found".to_string(),
            ))
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
                return Json(A2AResponse::error(
                    -32700,
                    "Unable to process your query. Please try rephrasing.".to_string(),
                ))
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

                return Json(A2AResponse::error(
                    -32600,
                    "Failed to fetch trending repositories. Try again later".to_string(),
                ))
                .into_response();
            }
        }
    };

    let response_text = format_trending_message(&repos, params);

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

    Json(response).into_response()
}
