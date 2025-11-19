use crate::Result;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

/// Simple structured request/response for Claude Haiku 3/4 (approximate).
/// This is a blocking client for simplicity. For production, use async and retries.

#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    messages: Vec<ClaudeMessage>,
    max_tokens: usize,
}

#[derive(Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    // adapt based on Anthropic/Claude response shape - this is a minimal schema
    completion: Option<String>,
    // ... add fields as needed
}

pub fn call_claude(prompt: &str) -> Result<String> {
    // Read API key
    let key = env::var("ANTHROPIC_API_KEY").or_else(|_| env::var("CLAUDE_API_KEY")).unwrap_or_default();
    if key.is_empty() {
        return Ok("LLM key not found. Set ANTHROPIC_API_KEY to enable LLM features.".to_string());
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(20))
        .build()?;

    // This is a generalized call — adapt to the exact Claude API shape you use.
    let body = ClaudeRequest {
        model: "claude-haiku-3".to_string(),
        messages: vec![ClaudeMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }],
        max_tokens: 400,
    };

    // Placeholder endpoint - update to Anthropic/Pinecone endpoint as appropriate
    let endpoint = env::var("CLAUDE_API_URL").unwrap_or_else(|_| "https://api.anthropic.com/v1/complete".to_string());

    let res = client
        .post(&endpoint)
        .bearer_auth(key)
        .json(&body)
        .send()?;

    let status = res.status();
    if !status.is_success() {
        let text = res.text().unwrap_or_default();
        return Ok(format!("LLM call failed: {} - {}", status, text));
    }

    let text = res.text().unwrap_or_default();
    // Simple: return raw text. You can parse JSON from Anthropic if desired.
    Ok(text)
}
