use anyhow::Result;
use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc, Weekday};

#[derive(Debug, Clone, PartialEq)]
pub struct DateRange {
    pub created_after: NaiveDate,
    pub pushed_after: NaiveDate,
}

pub struct DateParser;

impl DateParser {
    /// Supports formats:
    /// - Specific dates: "23 January 2013", "January 23, 2013", "2013-01-23"
    /// - Relative dates: "yesterday", "last week", "last Tuesday"
    /// - Timeframes: "day", "week", "month", "quarter", "year"
    pub fn parse(date_str: &str) -> Result<DateRange> {
        let normalized = date_str.trim().to_lowercase();

        Self::parse_relative_date(&normalized)
            .or_else(|| Self::parse_timeframe(&normalized))
            .or_else(|| Self::parse_specific_date(&normalized).map(Self::create_range_from_date))
            .ok_or_else(|| anyhow::anyhow!("Unrecognized date format"))
    }

    fn parse_relative_date(date_str: &str) -> Option<DateRange> {
        let now = Utc::now();

        match date_str {
            val if val.contains("yesterday") => {
                let date = (now - Duration::days(1)).date_naive();
                Some(Self::create_range_from_date(date))
            }
            val if val.contains("today") => {
                let date = now.date_naive();
                Some(Self::create_range_from_date(date))
            }
            val if val.contains("week") => Self::parse_timeframe("week"),
            val if val.contains("month") => Self::parse_timeframe("month"),
            val if val.contains("year") => Self::parse_timeframe("year"),
            val if val.contains("last") && val.contains("day") => {
                // Extract number: "last 30 days"
                let days = Self::extract_number_from_string(val).unwrap_or(7);
                let date = (now - Duration::days(days as i64)).date_naive();
                Some(Self::create_range_from_date(date))
            }
            val if val.contains("last") && Self::is_weekday(val) => {
                let weekday = Self::extract_weekday_from_string(val)?;
                let date = Self::get_last_weekday(now, weekday);
                Some(Self::create_range_from_date(date))
            }
            _ => None,
        }
    }

    fn parse_timeframe(timeframe: &str) -> Option<DateRange> {
        let now = Utc::now();

        let (created_days, pushed_days) = match timeframe {
            "day" => (1, 1),
            "week" => (7, 7),
            "month" => (30, 30),
            "quarter" => (90, 90),
            "year" => (365, 180),
            _ => return None,
        };

        let created_after = (now - Duration::days(created_days)).date_naive();
        let pushed_after = (now - Duration::days(pushed_days)).date_naive();

        Some(DateRange {
            created_after,
            pushed_after,
        })
    }

    fn parse_specific_date(date_str: &str) -> Option<NaiveDate> {
        let formats = vec![
            "%d %B %Y",  // "23 January 2013"
            "%B %d, %Y", // "January 23, 2013"
            "%Y-%m-%d",  // "2013-01-23"
            "%d-%m-%Y",  // "23-01-2013"
            "%m/%d/%Y",  // "01/23/2013"
            "%d/%m/%Y",  // "23/01/2013"
            "%B %d %Y",  // "January 23 2013" (no comma)
            "%d %b %Y",  // "23 Jan 2013" (abbreviated month)
            "%b %d, %Y", // "Jan 23, 2013"
        ];

        for format in formats {
            if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
                return Some(date);
            }
        }

        Self::parse_flexible_date(date_str)
    }

    fn parse_flexible_date(date_str: &str) -> Option<NaiveDate> {
        let cleaned = date_str
            .replace("after", "")
            .replace("since", "")
            .replace("from", "")
            .trim()
            .to_string();

        let year_regex = regex::Regex::new(r"\b(19|20)\d{2}\b").ok()?;
        let year = year_regex.find(&cleaned)?.as_str().parse::<i32>().ok()?;

        let month = Self::extract_month_from_string(&cleaned)?;

        let day = Self::extract_day_from_string(&cleaned)?;

        NaiveDate::from_ymd_opt(year, month, day)
    }

    fn extract_month_from_string(date_str: &str) -> Option<u32> {
        let months = vec![
            ("january", 1),
            ("jan", 1),
            ("february", 2),
            ("feb", 2),
            ("march", 3),
            ("mar", 3),
            ("april", 4),
            ("apr", 4),
            ("may", 5),
            ("june", 6),
            ("jun", 6),
            ("july", 7),
            ("jul", 7),
            ("august", 8),
            ("aug", 8),
            ("september", 9),
            ("sep", 9),
            ("sept", 9),
            ("october", 10),
            ("oct", 10),
            ("november", 11),
            ("nov", 11),
            ("december", 12),
            ("dec", 12),
        ];

        for (name, num) in months {
            if date_str.contains(name) {
                return Some(num);
            }
        }

        regex::Regex::new(r"\b(1[0-2]|0?[1-9])\b")
            .ok()?
            .find(date_str)?
            .as_str()
            .parse()
            .ok()
    }

    fn extract_day_from_string(date_str: &str) -> Option<u32> {
        // Look for numbers that could be days (1-31)
        let parts: Vec<&str> = date_str.split(|c: char| !c.is_numeric()).collect();
        for part in parts {
            if let Ok(day) = part.parse::<u32>() {
                if (1..=31).contains(&day) {
                    return Some(day);
                }
            }
        }
        None
    }

    fn extract_number_from_string(date_str: &str) -> Option<i64> {
        regex::Regex::new(r"\d+")
            .ok()?
            .find(date_str)?
            .as_str()
            .parse()
            .ok()
    }

    fn is_weekday(date_str: &str) -> bool {
        let weekdays = [
            "monday",
            "tuesday",
            "wednesday",
            "thursday",
            "friday",
            "saturday",
            "sunday",
        ];
        weekdays.iter().any(|&day| date_str.contains(day))
    }

    fn extract_weekday_from_string(date_str: &str) -> Option<Weekday> {
        use Weekday::*;

        [
            ("monday", Mon),
            ("tuesday", Tue),
            ("wednesday", Wed),
            ("thursday", Thu),
            ("friday", Fri),
            ("saturday", Sat),
            ("sunday", Sun),
        ]
        .iter()
        .find(|(name, _)| date_str.contains(*name))
        .map(|(_, day)| *day)
    }

    fn get_last_weekday(now: DateTime<Utc>, target_weekday: Weekday) -> NaiveDate {
        (1..=7)
            .map(|i| (now - Duration::days(i)).date_naive())
            .find(|date| date.weekday() == target_weekday)
            .unwrap_or_else(|| (now - Duration::days(7)).date_naive())
    }

    fn create_range_from_date(date: NaiveDate) -> DateRange {
        let now = Utc::now().date_naive();
        let days_ago = (now - date).num_days();

        let pushed_days = if days_ago <= 30 {
            days_ago
        } else if days_ago <= 90 {
            90
        } else {
            180
        };

        let pushed_after = now - Duration::days(pushed_days);

        DateRange {
            created_after: date,
            pushed_after,
        }
    }

    pub fn calculate_timeframe_from_date(date: NaiveDate) -> String {
        let now = Utc::now().date_naive();
        match (now - date).num_days() {
            days if days <= 1 => "day",
            days if days <= 7 => "week",
            days if days <= 30 => "month",
            days if days <= 90 => "quarter",
            _ => "year",
        }
        .to_string()
    }
}
