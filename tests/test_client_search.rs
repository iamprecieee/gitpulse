use std::env;

use anyhow::{Ok, Result};
use dotenvy::dotenv;
use gitpulse::{
    models::{query::QueryParams, repository::TrendingRepo},
    services::github::GitHubClient,
    utils::helpers::format_trending_message,
};

fn create_test_client() -> Result<GitHubClient> {
    dotenv().ok();

    let github_search_url = env::var("GITHUB_SEARCH_URL")?;
    GitHubClient::new(None, github_search_url.as_str())
}

#[test]
fn test_format_message() {
    let repos = vec![TrendingRepo {
        name: "test/repo".to_string(),
        description: "A test repo".to_string(),
        url: "https://github.com/test/repo".to_string(),
        language: "Rust".to_string(),
        stars: 100,
    }];

    let message = format_trending_message(&repos, "test");
    assert!(message.contains("test/repo"));
    assert!(message.contains("100 stars"));
}

async fn search_and_verify(
    params: QueryParams,
    expected_min_count: usize,
    description: &str,
) -> Result<()> {
    let client = create_test_client()?;

    let repos = client
        .search_with_params(&params)
        .await
        .expect("Search with params failed");

    assert!(
        repos.len() >= expected_min_count,
        "Should return at least {} {}",
        expected_min_count,
        description
    );

    if !repos.is_empty() {
        let first = &repos[0];
        println!("Found {} {}", repos.len(), description);
        println!("Top repo: {} ({} stars)", first.name, first.stars);
    }

    Ok(())
}

#[tokio::test]
async fn test_search_with_params() -> Result<()> {
    let params = QueryParams {
        language: Some("rust".to_string()),
        topics: vec![],
        timeframe: "week".to_string(),
        count: 3,
        min_stars: 10,
        date_string: None,
        created_after: None,
        pushed_after: None,
        has_specific_date: false,
    };

    search_and_verify(params, 1, "Rust repos").await
}

#[tokio::test]
async fn test_search_with_topics() -> Result<()> {
    let params = QueryParams {
        language: Some("python".to_string()),
        topics: vec!["machine-learning".to_string(), "ai".to_string()],
        timeframe: "month".to_string(),
        count: 5,
        min_stars: 50,
        date_string: None,
        created_after: None,
        pushed_after: None,
        has_specific_date: false,
    };

    search_and_verify(params, 1, "Python AI/ML repos").await
}

#[tokio::test]
async fn test_search_with_invalid_topics() -> Result<()> {
    let params = QueryParams {
        language: Some("python".to_string()),
        topics: vec!["machine-learning".to_string(), "heeheehee".to_string()],
        timeframe: "month".to_string(),
        count: 5,
        min_stars: 50,
        date_string: None,
        created_after: None,
        pushed_after: None,
        has_specific_date: false,
    };

    search_and_verify(params, 1, "Python repos (with invalid topic)").await
}
