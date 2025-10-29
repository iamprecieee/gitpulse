use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryParams {
    pub language: Option<String>,
    #[serde(default)]
    pub topics: Vec<String>,
    #[serde(default = "default_timeframe")]
    pub timeframe: String,
    #[serde(default = "default_count")]
    pub count: usize,
    #[serde(default = "default_min_stars")]
    pub min_stars: u32,
}

fn default_timeframe() -> String {
    "week".to_string()
}

fn default_count() -> usize {
    5
}

fn default_min_stars() -> u32 {
    10
}

impl Default for QueryParams {
    fn default() -> Self {
        Self {
            language: None,
            topics: vec![],
            timeframe: default_timeframe(),
            count: default_count(),
            min_stars: default_min_stars(),
        }
    }
}
