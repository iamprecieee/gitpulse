use anyhow::Result;
use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};

#[derive(Debug, Clone, PartialEq)]
pub struct DateRange {
    pub created_after: NaiveDate,
    pub pushed_after: NaiveDate,
}

pub struct DateParser;

impl DateParser {
    /// Parse a date string into a DateRange
    ///
    /// Supports formats:
    /// - Specific dates: "23 January 2013", "January 23, 2013", "2013-01-23"
    /// - Relative dates: "yesterday", "last week", "last Tuesday"
    /// - Timeframes: "day", "week", "month", "quarter", "year"
    pub fn parse(date_str: &str) -> Result<DateRange> {
        let normalized = date_str.trim().to_lowercase();

        if let Some(range) = Self::parse_relative_date(&normalized) {
            return Ok(range);
        }

        if let Some(range) = Self::parse_timeframe(&normalized) {
            return Ok(range);
        }

        if let Some(date) = Self::parse_specific_date(&normalized) {
            return Ok(Self::create_range_from_date(date));
        }

        Ok(Self::parse_timeframe("week").unwrap())
    }

    /// Parse relative date expressions like "yesterday", "last week"
    fn parse_relative_date(date_str: &str) -> Option<DateRange> {
        let now = Utc::now();

        match date_str {
            s if s.contains("yesterday") => {
                let date = (now - Duration::days(1)).date_naive();
                Some(Self::create_range_from_date(date))
            }
            s if s.contains("today") => {
                let date = now.date_naive();
                Some(Self::create_range_from_date(date))
            }
            s if s.contains("last week") || s.contains("past week") || s.contains("this week") => {
                Self::parse_timeframe("week")
            }
            s if s.contains("last month")
                || s.contains("past month")
                || s.contains("this month") =>
            {
                Self::parse_timeframe("month")
            }
            s if s.contains("last year") || s.contains("past year") || s.contains("this year") => {
                Self::parse_timeframe("year")
            }
            s if s.contains("last") && s.contains("day") => {
                // Extract number: "last 30 days"
                let days = Self::extract_number(s).unwrap_or(7);
                let date = (now - Duration::days(days as i64)).date_naive();
                Some(Self::create_range_from_date(date))
            }
            s if s.contains("last") && Self::is_weekday(s) => {
                // "last Tuesday", "last Friday"
                let weekday = Self::extract_weekday(s)?;
                let date = Self::get_last_weekday(now, weekday);
                Some(Self::create_range_from_date(date))
            }
            _ => None,
        }
    }

    /// Parse timeframe keywords
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

    /// Parse specific date formats
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

    /// Flexible date parsing for various formats
    fn parse_flexible_date(date_str: &str) -> Option<NaiveDate> {
        let cleaned = date_str
            .replace("after", "")
            .replace("since", "")
            .replace("from", "")
            .trim()
            .to_string();

        if let Some((year, month, day)) = Self::extract_date_components(&cleaned) {
            return NaiveDate::from_ymd_opt(year, month, day);
        }

        None
    }

    /// Extract date components from a string
    fn extract_date_components(s: &str) -> Option<(i32, u32, u32)> {
        let year_regex = regex::Regex::new(r"\b(19|20)\d{2}\b").ok()?;
        let year = year_regex.find(s)?.as_str().parse::<i32>().ok()?;

        let month = Self::extract_month(s)?;

        let day = Self::extract_day(s)?;

        Some((year, month, day))
    }

    /// Extract month from string
    fn extract_month(s: &str) -> Option<u32> {
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
            if s.contains(name) {
                return Some(num);
            }
        }

        // Try numeric month
        let parts: Vec<&str> = s.split(|c: char| !c.is_numeric()).collect();
        for part in parts {
            if let Ok(month) = part.parse::<u32>() {
                if (1..=12).contains(&month) {
                    return Some(month);
                }
            }
        }

        None
    }

    /// Extract day from string
    fn extract_day(s: &str) -> Option<u32> {
        // Look for numbers that could be days (1-31)
        let parts: Vec<&str> = s.split(|c: char| !c.is_numeric()).collect();
        for part in parts {
            if let Ok(day) = part.parse::<u32>() {
                if (1..=31).contains(&day) {
                    return Some(day);
                }
            }
        }
        None
    }

    /// Extract a number from a string
    fn extract_number(s: &str) -> Option<i64> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        for part in parts {
            if let Ok(num) = part.parse::<i64>() {
                return Some(num);
            }
        }
        None
    }

    /// Check if string contains a weekday name
    fn is_weekday(s: &str) -> bool {
        let weekdays = [
            "monday",
            "tuesday",
            "wednesday",
            "thursday",
            "friday",
            "saturday",
            "sunday",
        ];
        weekdays.iter().any(|&day| s.contains(day))
    }

    /// Extract weekday from string
    fn extract_weekday(s: &str) -> Option<chrono::Weekday> {
        use chrono::Weekday;

        if s.contains("monday") {
            Some(Weekday::Mon)
        } else if s.contains("tuesday") {
            Some(Weekday::Tue)
        } else if s.contains("wednesday") {
            Some(Weekday::Wed)
        } else if s.contains("thursday") {
            Some(Weekday::Thu)
        } else if s.contains("friday") {
            Some(Weekday::Fri)
        } else if s.contains("saturday") {
            Some(Weekday::Sat)
        } else if s.contains("sunday") {
            Some(Weekday::Sun)
        } else {
            None
        }
    }

    /// Get the most recent occurrence of a weekday
    fn get_last_weekday(now: DateTime<Utc>, target_weekday: chrono::Weekday) -> NaiveDate {
        let mut date = now.date_naive();

        // Go back up to 7 days to find the target weekday
        for _ in 0..7 {
            date = date - Duration::days(1);
            if date.weekday() == target_weekday {
                return date;
            }
        }

        (now - Duration::days(7)).date_naive()
    }

    /// Create a DateRange from a specific date
    fn create_range_from_date(date: NaiveDate) -> DateRange {
        let now = Utc::now().date_naive();
        let days_ago = (now - date).num_days();

        // Use the same logic as timeframe calculation
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

    /// Calculate timeframe string from a specific date (for backwards compatibility)
    pub fn calculate_timeframe_from_date(date: NaiveDate) -> String {
        let now = Utc::now().date_naive();
        let days_ago = (now - date).num_days();

        if days_ago <= 1 {
            "day".to_string()
        } else if days_ago <= 7 {
            "week".to_string()
        } else if days_ago <= 30 {
            "month".to_string()
        } else if days_ago <= 90 {
            "quarter".to_string()
        } else {
            "year".to_string()
        }
    }
}
