use std::{
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    body::Body,
    extract::{ConnectInfo, Request},
    http::HeaderValue,
    middleware::Next,
    response::Response,
};

use dashmap::DashMap;
use reqwest::StatusCode;
use tokio::time::interval;

#[derive(Clone)]
pub struct RateLimiter {
    windows: Arc<DashMap<String, Window>>,
    requests_per_window: u32,
    window_duration: Duration,
}

#[derive(Debug)]
struct Window {
    window_start: Instant,
    request_count: u32,
}

impl RateLimiter {
    pub fn new(requests_per_window: u32, window_secs: u64) -> Self {
        let limiter = Self {
            windows: Arc::new(DashMap::new()),
            requests_per_window,
            window_duration: Duration::from_secs(window_secs),
        };

        let windows = limiter.windows.clone();
        let ttl = limiter.window_duration * 2;
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                let now = Instant::now();
                windows.retain(|_, entry| now.duration_since(entry.window_start) < ttl);
            }
        });

        limiter
    }

    pub fn check_rate_limit(&self, client_key: &str) -> bool {
        let now = Instant::now();

        let mut entry = self
            .windows
            .entry(client_key.to_string())
            .or_insert(Window {
                window_start: now,
                request_count: 0,
            });

        if now.duration_since(entry.window_start) >= self.window_duration {
            entry.window_start = now;
            entry.request_count = 0;
        }

        if entry.request_count >= self.requests_per_window {
            return false;
        }

        entry.request_count += 1;
        true
    }
}

pub async fn rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let rate_limiter = req
        .extensions()
        .get::<RateLimiter>()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let client_key = addr.ip().to_string();

    if !rate_limiter.check_rate_limit(&client_key) {
        let mut res = Response::new(Body::empty());
        *res.status_mut() = StatusCode::TOO_MANY_REQUESTS;
        res.headers_mut()
            .insert("Retry-After", HeaderValue::from_static("60"));
        return Ok(res);
    }

    Ok(next.run(req).await)
}
