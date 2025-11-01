use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::models::{
    a2a::{A2ARequest, Artifact, MessagePart},
    query::QueryParams,
    repository::TrendingRepo,
};

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
    let data_part = request
        .params
        .message
        .parts
        .iter()
        .find(|part| matches!(part, MessagePart::Data { .. }))?;

    if let MessagePart::Data { data, .. } = data_part {
        let len = data.len();
        if len < 2 {
            return None;
        }

        if let Some(user_msg) = data.get(len - 2) {
            if let Some(text) = user_msg.get("text").and_then(|v| v.as_str()) {
                let cleaned = text
                    .replace("<p>", "")
                    .replace("</p>", "")
                    .replace("<br />", "")
                    .trim()
                    .to_string();

                if !cleaned.is_empty() {
                    return Some(cleaned);
                }
            }
        }
    }

    request
        .params
        .message
        .parts
        .iter()
        .find(|part| matches!(part, MessagePart::Text { .. }))
        .and_then(|part| match part {
            MessagePart::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
}

pub fn create_artifacts(response_text: String) -> Vec<Artifact> {
    let mut artifacts = Vec::new();

    artifacts.push(Artifact {
        artifact_id: Uuid::new_v4().to_string(),
        name: "gitpulseAgentResponse".to_string(),
        parts: vec![MessagePart::Text {
            kind: "text".to_string(),
            text: response_text,
        }],
    });

    artifacts
}
