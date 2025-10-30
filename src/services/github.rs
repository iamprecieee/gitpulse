use std::collections::HashSet;

use anyhow::{Context, Result};
use axum::http::{HeaderMap, HeaderValue};
use reqwest::header::{ACCEPT, USER_AGENT};

use crate::{
    models::{
        query::QueryParams,
        repository::{SearchResponse, TrendingRepo},
    },
    utils::helpers::calculate_date_filters,
};

#[derive(Debug, Clone)]
pub struct GitHubClient {
    client: reqwest::Client,
    search_url: String,
}

impl GitHubClient {
    pub fn new(github_access_token: Option<String>, github_search_url: String) -> Result<Self> {
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
            search_url: github_search_url,
        })
    }

    pub async fn search_with_params(&self, params: &QueryParams) -> Result<Vec<TrendingRepo>> {
        let base_query_parts = self.build_base_query_parts(params);

        if !params.topics.is_empty() {
            if let Some(repos) = self.try_search_all_topics(&base_query_parts, params).await {
                return Ok(repos);
            }

            if let Some(repos) = self
                .search_topics_individually(&base_query_parts, params)
                .await
            {
                return Ok(repos);
            }
        }

        let query = base_query_parts.join("+");

        tracing::info!("GitHub search query (no topics): {}", query);

        self.search_repositories(&query, params.count).await
    }

    async fn try_search_all_topics(
        &self,
        base_query_parts: &[String],
        params: &QueryParams,
    ) -> Option<Vec<TrendingRepo>> {
        let mut all_topics_parts = base_query_parts.to_vec();

        for topic in &params.topics {
            all_topics_parts.insert((all_topics_parts.len() - 1) as usize, topic.clone());
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

    async fn search_topics_individually(
        &self,
        base_query_parts: &[String],
        params: &QueryParams,
    ) -> Option<Vec<TrendingRepo>> {
        let mut all_repos = Vec::new();
        let mut seen_names = HashSet::new();

        for topic in &params.topics {
            if let Some(repos) = self
                .search_single_topic(base_query_parts, topic, params.count)
                .await
            {
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

    async fn search_single_topic(
        &self,
        base_query_parts: &[String],
        topic: &str,
        count: usize,
    ) -> Option<Vec<TrendingRepo>> {
        let mut single_topic_parts = base_query_parts.to_vec();
        single_topic_parts.insert((single_topic_parts.len() - 1) as usize, topic.to_string());

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

    pub fn format_trending_message(repos: &[TrendingRepo], timeframe: &str) -> String {
        if repos.is_empty() {
            return format!("No trending repositories found for {}.", timeframe);
        }

        let mut message = format!("Trending on GitHub ({})\n\n", timeframe);

        for (i, repo) in repos.iter().enumerate() {
            message.push_str(&format!(
                "{}. {} - {} stars\n   {} - {}\n   {}\n\n",
                i + 1,
                repo.name,
                repo.stars,
                repo.language,
                repo.description,
                repo.url
            ));
        }

        message
    }

    fn build_base_query_parts(&self, params: &QueryParams) -> Vec<String> {
        let (created_date, pushed_date) = calculate_date_filters(&params.timeframe);

        let mut query_parts = vec![
            format!("created:>{}", created_date),
            format!("pushed:>{}", pushed_date),
        ];

        if let Some(ref language) = params.language {
            query_parts.push(format!("language:{}", language));
        }

        if params.min_stars > 0 {
            query_parts.push(format!("stars:>={}", params.min_stars));
        }

        query_parts
    }
}
