use chrono::{Duration, Utc};

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
