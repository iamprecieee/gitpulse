use axum::{
    Router,
    routing::{get, post},
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    api::{
        routes::{get_trending, health_check},
        state::AppState,
    },
    models::a2a::{
        A2ARequest, A2AResponse, Configuration, ErrorDetail, Message, MessagePart, RequestParams,
        TaskResult, TaskStatus,
    },
};

pub mod routes;
pub mod state;

#[derive(OpenApi)]
#[openapi(
    paths(crate::api::routes::health_check, crate::api::routes::get_trending,),
    components(schemas(
        A2ARequest,
        RequestParams,
        Message,
        Configuration,
        MessagePart,
        A2AResponse,
        TaskResult,
        TaskStatus,
        ErrorDetail
    )),
    info(title = "GitPulse API", version = "1.0.0")
)]
pub struct ApiDoc;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/trending", post(get_trending))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}
