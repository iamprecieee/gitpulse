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
    let (created_date, pushed_date) = if params.uses_specific_dates() {
        (
            params.created_after.clone().unwrap(),
            params.pushed_after.clone().unwrap(),
        )
    } else {
        calculate_date_filters(&params.timeframe)
    };

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

pub fn format_trending_message(repos: &[TrendingRepo], params: QueryParams) -> String {
    let timeframe = match params.has_specific_date {
        true => params.created_after.unwrap_or(params.timeframe),
        false => params.timeframe,
    };

    if repos.is_empty() {
        return format!("No trending repositories found for {}.", timeframe);
    }

    let mut message = String::new();

    message.push_str("**TRENDING ON GITHUB**\n\n");
    message.push_str(&format!("**PERIOD:** {}\n\n", timeframe));
    message.push_str("---\n\n");

    for (i, repo) in repos.iter().enumerate() {
        let stars = format_number(repo.stars);

        message.push_str(&format!(
            "### {}. - [{}]({})\n",
            i + 1,
            repo.name,
            repo.url
        ));

        message.push_str(&format!(">> {}\n", repo.description));

        message.push_str(&format!(
            "**STARS:** {} | **LANGUAGE:** {}\n",
            stars, repo.language
        ));

        if i < repos.len() - 1 {
            message.push_str("---\n");
        }
    }

    message.push_str(&format!(
        "\n**_Found {} trending repositories_**\n",
        repos.len()
    ));

    message
}

fn format_number(num: u32) -> String {
    if num >= 1_000_000 {
        format!("{:.1}M", num as f64 / 1_000_000.0)
    } else if num >= 1_000 {
        format!("{:.1}k", num as f64 / 1_000.0)
    } else {
        num.to_string()
    }
}

pub fn extract_user_query(request: &A2ARequest) -> Option<String> {
    let data_part = request
        .params
        .message
        .parts
        .iter()
        .find(|part| matches!(part, MessagePart::Data { .. }));

    if let Some(MessagePart::Data { data, .. }) = data_part {
        for entry in data.iter().rev() {
            if let Some(text) = entry.get("text").and_then(|v| v.as_str()) {
                let trimmed = text.trim();

                let is_user_query = trimmed.starts_with("<p>") || trimmed.len() > 0;

                if is_user_query {
                    let cleaned = trimmed
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
    }

    for part in &request.params.message.parts {
        if let MessagePart::Text { text, .. } = part {
            let cleaned = text.trim();
            if !cleaned.is_empty() {
                return Some(cleaned.to_string());
            }
        }
    }

    None
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
