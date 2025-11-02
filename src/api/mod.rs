use axum::{
    Extension, Router,
    http::{self, HeaderValue},
    middleware,
    routing::{get, post},
};
use tower_http::cors::CorsLayer;
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
    services::rate_limiter::rate_limit_middleware,
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
    let cors = CorsLayer::new()
        .allow_origin(
            state
                .config
                .cors_allowed_origins
                .split(',')
                .map(|val| val.trim())
                .filter(|val| !val.is_empty())
                .filter_map(|origin| origin.parse::<HeaderValue>().ok())
                .collect::<Vec<_>>(),
        )
        .allow_methods([http::Method::GET, http::Method::POST, http::Method::OPTIONS])
        .allow_headers([
            http::header::CONTENT_TYPE,
            http::header::COOKIE,
            http::header::CACHE_CONTROL,
        ])
        .allow_credentials(true);

    let api_routes = Router::new()
        .route("/health", get(health_check))
        .route("/trending", post(get_trending))
        .layer(middleware::from_fn(rate_limit_middleware))
        .layer(Extension(state.rate_limiter.clone()))
        .layer(cors);

    Router::new()
        .merge(api_routes)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}
