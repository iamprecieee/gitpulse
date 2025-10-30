use std::sync::Arc;

use anthropic_sdk::{Anthropic, ContentBlock, MessageCreateBuilder};
use anyhow::{Context, Result};
use google_ai_rs::Client;

use crate::models::query::QueryParams;

#[derive(Clone)]
enum LlmClient {
    Claude(Arc<Anthropic>),
    Gemini(Client),
}

#[derive(Clone)]
pub struct QueryParser {
    client: LlmClient,
    model: String,
    system_prompt: String,
}

impl QueryParser {
    pub async fn new(
        llm_provider: &str,
        api_key: &str,
        model: &str,
        system_prompt: &str,
    ) -> Result<Self> {
        let client = if llm_provider == "anthropic" {
            let anthropic = Arc::new(Anthropic::new(api_key)?);

            LlmClient::Claude(anthropic)
        } else if llm_provider == "gemini" {
            let gemini = Client::new(api_key)
                .await
                .context("Failed to initialize Gemini client")?;

            LlmClient::Gemini(gemini)
        } else {
            anyhow::bail!(
                "Unknown provider: {}. Must be either 'claude' or 'gemini'",
                model
            );
        };

        Ok(Self {
            client,
            model: model.to_string(),
            system_prompt: system_prompt.to_string(),
        })
    }

    pub async fn parse(&self, user_query: &str) -> Result<QueryParams> {
        let response_text = match &self.client {
            LlmClient::Claude(anthropic_client) => {
                let response = anthropic_client
                    .messages()
                    .create(
                        MessageCreateBuilder::new(&self.model, 200)
                            .system(&self.system_prompt)
                            .user(user_query)
                            .build(),
                    )
                    .await
                    .context("Failed to call Anthropic API")?;

                response
                    .content
                    .into_iter()
                    .filter_map(|block| match block {
                        ContentBlock::Text { text } => Some(text),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("")
            }
            LlmClient::Gemini(gemini_client) => {
                let full_prompt = format!("{}\n\nQuery: \"{}\"", self.system_prompt, user_query);

                let model = gemini_client.generative_model(&self.model);

                let response = model
                    .generate_content(full_prompt)
                    .await
                    .context("Failed to call Gemini API")?;

                response.text()
            }
        };

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
