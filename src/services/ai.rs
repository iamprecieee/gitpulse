use anyhow::{Context, Result};
use google_ai_rs::Client;

use crate::models::query::QueryParams;

#[derive(Clone)]
pub struct QueryParser {
    client: Client,
    model: String,
    system_prompt: String,
}

impl QueryParser {
    pub async fn new(api_key: &str, model: &str, system_prompt: &str) -> Result<Self> {
        let client = Client::new(api_key)
            .await
            .context("Failed to initialize Gemini client")?;

        Ok(Self {
            client,
            model: model.to_string(),
            system_prompt: system_prompt.to_string(),
        })
    }

    pub async fn parse(&self, user_query: &str) -> Result<QueryParams> {
        let full_prompt = format!("{}\n\nQuery: \"{}\"", self.system_prompt, user_query);

        let model = self.client.generative_model(&self.model);

        let response = model
            .generate_content(full_prompt)
            .await
            .context("Failed to call Gemini API")?;

        let response_text = response.text();

        tracing::debug!(target: "Gemini raw", response = ?response_text);

        let params = self.parse_llm_response(&response_text)?;

        Ok(params)
    }

    fn parse_llm_response(&self, response_text: &str) -> Result<QueryParams> {
        let cleaned = response_text
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let params: QueryParams = match serde_json::from_str(cleaned) {
            Ok(params) => params,
            Err(e) => {
                tracing::warn!("LLM response parsing failed: {e}, falling back to defaults");
                QueryParams::default()
            }
        };

        Ok(params)
    }
}
