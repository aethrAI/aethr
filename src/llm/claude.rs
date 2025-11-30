use crate::Result;
use serde::{Deserialize, Serialize};

const CLAUDE_API_URL: &str = "https://api.anthropic.com/v1/messages";
const MODEL: &str = "claude-sonnet-4-20250514";

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
    system: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ContentBlock>,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    text: String,
}

pub struct ClaudeClient {
    api_key: String,
    client: reqwest::blocking::Client,
}

impl ClaudeClient {
    /// Create a new Claude client from environment variable
    pub fn from_env() -> Option<Self> {
        let api_key = std::env::var("ANTHROPIC_API_KEY").ok()?;
        if api_key.is_empty() {
            return None;
        }
        Some(Self {
            api_key,
            client: reqwest::blocking::Client::new(),
        })
    }

    /// Get a fix suggestion for a terminal error
    pub fn get_fix(&self, error: &str, context: Option<&str>) -> Result<FixSuggestion> {
        let system = r#"You are a terminal error fixing assistant. Given an error message, provide a concise fix.

Rules:
- Return ONLY the command(s) to fix the issue, one per line
- If multiple steps needed, number them
- Keep explanations very brief (one sentence max)
- Focus on the most likely fix first
- Consider the project context if provided

Format your response EXACTLY like this:
COMMAND: <the fix command>
EXPLANATION: <brief one-sentence explanation>"#;

        let user_message = if let Some(ctx) = context {
            format!("Error: {}\n\nProject context: {}", error, ctx)
        } else {
            format!("Error: {}", error)
        };

        let request = ClaudeRequest {
            model: MODEL.to_string(),
            max_tokens: 300,
            system: system.to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: user_message,
            }],
        };

        let response = self
            .client
            .post(CLAUDE_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            anyhow::bail!("Claude API error ({}): {}", status, body);
        }

        let claude_response: ClaudeResponse = response.json()?;
        
        let text = claude_response
            .content
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default();

        // Parse the response
        let mut command = String::new();
        let mut explanation = String::new();

        for line in text.lines() {
            let line = line.trim();
            if line.starts_with("COMMAND:") {
                command = line.strip_prefix("COMMAND:").unwrap_or("").trim().to_string();
            } else if line.starts_with("EXPLANATION:") {
                explanation = line.strip_prefix("EXPLANATION:").unwrap_or("").trim().to_string();
            }
        }

        // Fallback: if parsing failed, use first line as command
        if command.is_empty() && !text.is_empty() {
            command = text.lines().next().unwrap_or("").trim().to_string();
        }

        Ok(FixSuggestion { command, explanation })
    }
}

#[derive(Debug)]
pub struct FixSuggestion {
    pub command: String,
    pub explanation: String,
}
