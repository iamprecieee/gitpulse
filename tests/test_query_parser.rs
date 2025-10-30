use anyhow::Result;
use dotenvy::dotenv;
use gitpulse::services::ai::QueryParser;
use std::{env, fs};

#[tokio::test]
async fn test_parse_query_with_real_gemini_api() -> Result<()> {
    dotenv().ok();

    let api_key = env::var("LLM_API_KEY")?;
    let model = env::var("LLM_MODEL")?;
    let system_prompt =
        fs::read_to_string("system_prompt.txt").expect("Failed to load system prompt");

    let parser = QueryParser::new(api_key.as_str(), model.as_str(), system_prompt.as_str()).await?;

    let user_query = "Get trending AI and Biotech repositories written in Rust created after October 1st 2025. add natural lang too";

    let params = parser.parse(user_query).await?;

    println!("Parsed parameters: {:?}", params);

    assert!(params.language.unwrap().to_lowercase().contains("rust"));
    assert_eq!(params.topics.iter().count(), 3);
    assert_eq!(params.timeframe, "week".to_string());
    assert_eq!(params.min_stars, 10u32);

    Ok(())
}
