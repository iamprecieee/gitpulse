use chrono::{Duration, Utc};

use crate::models::{a2a::A2ARequest, query::QueryParams, repository::TrendingRepo};

pub fn calculate_date_filters(timeframe: &String) -> (String, String) {
    let created_days = match timeframe.as_str() {
        "day" => 1,
        "week" => 7,
        "month" => 30,
        "quarter" => 90,
        "year" => 365,
        _ => 7,
    };

    let pushed_days = match timeframe.as_str() {
        "day" | "week" | "month" => created_days,
        "quarter" => 90,
        "year" => 180,
        _ => created_days,
    };

    let now = Utc::now();
    let created_date = (now - Duration::days(created_days))
        .format("%Y-%m-%d")
        .to_string();
    let pushed_date = (now - Duration::days(pushed_days))
        .format("%Y-%m-%d")
        .to_string();

    (created_date, pushed_date)
}

pub fn build_base_query_parts(params: &QueryParams) -> Vec<String> {
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

pub fn extract_user_query(request: &A2ARequest) -> Option<String> {
    request
        .params
        .message
        .parts
        .first()
        .map(|part| part.text.clone())
}
