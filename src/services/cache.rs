use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use dashmap::DashMap;

use crate::models::{query::QueryParams, repository::TrendingRepo};

#[derive(Clone)]
struct CachedValue {
    repos: Option<Vec<TrendingRepo>>,
    params: Option<QueryParams>,
    cached_at: Instant,
}

#[derive(Clone)]
pub struct Cache {
    store: Arc<DashMap<String, CachedValue>>,
    ttl: Duration,
}

impl Cache {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            store: Arc::new(DashMap::new()),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    fn make_repo_key(params: &QueryParams) -> String {
        let mut sorted_topics = params.topics.clone();
        sorted_topics.sort();
        format!(
            "{}:{}:{}:{}:{}:{}:{}:{}",
            params.language.as_deref().unwrap_or("None"),
            sorted_topics.join(","),
            params.timeframe,
            params.count,
            params.min_stars,
            params.date_string.as_deref().unwrap_or("None"),
            params.created_after.as_deref().unwrap_or("None"),
            params.pushed_after.as_deref().unwrap_or("None"),
        )
    }

    fn make_llm_key(query: &str) -> String {
        query.trim().to_lowercase()
    }

    pub fn get_repo(&self, params: &QueryParams) -> Option<Vec<TrendingRepo>> {
        let key = Self::make_repo_key(params);

        if let Some(entry) = self.store.get(&key) {
            let elapsed = entry.cached_at.elapsed();

            if elapsed < self.ttl {
                tracing::info!("Cache HIT: {} (age: {}s)", key, elapsed.as_secs());
                return entry.repos.clone();
            } else {
                tracing::info!("Cache EXPIRED: {} (age: {}s)", key, elapsed.as_secs());
                drop(entry);
                self.store.remove(&key);
            }
        } else {
            tracing::info!("Cache MISS: {}", key);
        }

        None
    }

    pub fn get_llm(&self, query: &str) -> Option<QueryParams> {
        let key = Self::make_llm_key(query);

        if let Some(entry) = self.store.get(&key) {
            let elapsed = entry.cached_at.elapsed();

            if elapsed < self.ttl {
                tracing::info!("Cache HIT: '{}' (age: {}s)", query, elapsed.as_secs());
                return entry.params.clone();
            } else {
                tracing::info!(
                    "LLM Cache EXPIRED: '{}' (age: {}s)",
                    query,
                    elapsed.as_secs()
                );
                drop(entry);
                self.store.remove(&key);
            }
        } else {
            tracing::info!("Cache MISS: '{}'", query);
        }

        None
    }

    pub fn set(&self, query: Option<&str>, params: &QueryParams, repos: Option<Vec<TrendingRepo>>) {
        let (key, cached) = match repos {
            Some(val) => {
                let key = Self::make_repo_key(params);
                let repos = Some(val);

                let cached = CachedValue {
                    repos,
                    cached_at: Instant::now(),
                    params: None,
                };

                (key, cached)
            }
            None => match query {
                Some(val) => {
                    let key = Self::make_llm_key(val);
                    let params = Some(params.clone());

                    let cached = CachedValue {
                        params,
                        cached_at: Instant::now(),
                        repos: None,
                    };

                    (key, cached)
                }
                None => return (),
            },
        };

        self.store.insert(key.clone(), cached);
        tracing::info!("Cache SET: {}", key);
    }

    pub fn clear(&self) {
        self.store.clear();
        tracing::info!("Cache cleared");
    }
}
