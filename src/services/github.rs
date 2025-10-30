use std::collections::HashSet;

use anyhow::{Context, Result};
use axum::http::{HeaderMap, HeaderValue};
use futures::future::join_all;
use reqwest::header::{ACCEPT, USER_AGENT};

use crate::{
    models::{
        query::QueryParams,
        repository::{SearchResponse, TrendingRepo},
    },
    utils::helpers::build_base_query_parts,
};

#[derive(Debug, Clone)]
pub struct GitHubClient {
    client: reqwest::Client,
    search_url: String,
}

impl GitHubClient {
    pub fn new(github_access_token: Option<&str>, github_search_url: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();

        headers.insert(USER_AGENT, HeaderValue::from_static("gitpulse-agent"));
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github.v3+json"),
        );

        if let Some(token) = github_access_token {
            let auth_value = format!("Bearer {}", token);

            headers.insert(
                "Authorization",
                HeaderValue::from_str(&auth_value.as_str())
                    .context("Invalid GitHub access token format")?,
            );
        } else {
            tracing::warn!("No GitHub token provided - using unauthenticated requests");
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            client,
            search_url: github_search_url.to_string(),
        })
    }

    pub async fn search_with_params(&self, params: &QueryParams) -> Result<Vec<TrendingRepo>> {
        let base_query_parts = build_base_query_parts(params);

        if !params.topics.is_empty() {
            if let Some(repos) = self.search_all_topics(&base_query_parts, params).await {
                return Ok(repos);
            }

            if let Some(repos) = self.search_single_topic(&base_query_parts, params).await {
                return Ok(repos);
            }
        }

        let query = base_query_parts.join("+");

        tracing::info!("GitHub search query (no topics): {}", query);

        self.search_repositories(&query, params.count).await
    }

    async fn search_all_topics(
        &self,
        base_query_parts: &[String],
        params: &QueryParams,
    ) -> Option<Vec<TrendingRepo>> {
        let mut all_topics_parts = base_query_parts.to_vec();

        for topic in &params.topics {
            all_topics_parts.insert(0, topic.clone());
        }

        let query = all_topics_parts.join("+");

        tracing::info!("GitHub search query (all topics): {}", query);

        match self.search_repositories(&query, params.count).await {
            Ok(repos) if !repos.is_empty() => {
                tracing::info!("Found {} repos with all topics", repos.len());
                Some(repos)
            }
            Ok(_) => {
                tracing::warn!("No results with all topics, trying individual topics");
                None
            }
            Err(e) => {
                tracing::warn!(
                    "Error searching with all topics: {}, trying individual topics",
                    e
                );
                None
            }
        }
    }

    async fn search_single_topic(
        &self,
        base_query_parts: &[String],
        params: &QueryParams,
    ) -> Option<Vec<TrendingRepo>> {
        let search_futures = params
            .topics
            .iter()
            .map(|topic| self.search_topic(base_query_parts, topic, params.count));

        let results = join_all(search_futures).await;

        let mut all_repos = Vec::new();
        let mut seen_names = HashSet::new();

        for result in results {
            if let Some(repos) = result {
                for repo in repos {
                    if seen_names.insert(repo.name.clone()) {
                        all_repos.push(repo);
                    }
                }
            }
        }

        if all_repos.is_empty() {
            return None;
        }

        all_repos.sort_by(|a, b| b.stars.cmp(&a.stars));
        all_repos.truncate(params.count);

        tracing::info!(
            "Returning {} unique repos from individual topic searches",
            all_repos.len()
        );
        Some(all_repos)
    }

    async fn search_topic(
        &self,
        base_query_parts: &[String],
        topic: &str,
        count: usize,
    ) -> Option<Vec<TrendingRepo>> {
        let mut single_topic_parts = base_query_parts.to_vec();
        single_topic_parts.insert(0, topic.to_string());

        let query = single_topic_parts.join("+");

        tracing::info!("GitHub search query (topic: {}): {}", topic, query);

        match self.search_repositories(&query, count).await {
            Ok(repos) => {
                tracing::info!("Found {} repos for topic '{}'", repos.len(), topic);
                Some(repos)
            }
            Err(e) => {
                tracing::warn!("Failed to search for topic '{}': {}", topic, e);
                None
            }
        }
    }

    async fn search_repositories(&self, query: &str, limit: usize) -> Result<Vec<TrendingRepo>> {
        let url = format!(
            "{}?q={}&sort=stars&order=desc&per_page={}",
            self.search_url, query, limit
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send request to GitHub")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API error ({}): {}", status, error_text);
        }

        let response_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        let search_response: SearchResponse = serde_json::from_str(&response_text)
            .context("Failed to parse GitHub response as JSON")?;

        tracing::info!(
            "GitHub returned {} total results, {} items",
            search_response.total_count,
            search_response.items.len()
        );

        let trending_repos: Vec<TrendingRepo> = search_response
            .items
            .into_iter()
            .map(TrendingRepo::from)
            .collect();

        Ok(trending_repos)
    }
}
