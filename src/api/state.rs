use crate::{
    config::settings::Config,
    services::{ai::QueryParser, cache::Cache, github::GitHubClient, rate_limiter::RateLimiter},
};

#[derive(Clone)]
pub struct AppState {
    pub github_client: GitHubClient,
    pub config: Config,
    pub query_parser: QueryParser,
    pub cache: Cache,
    pub rate_limiter: RateLimiter,
}
