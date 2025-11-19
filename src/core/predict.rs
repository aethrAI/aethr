use crate::Result;
use crate::utils::api_client;

/// High-level predict wrapper used by CLI or daemon.
/// Returns raw model output (string). Later we can add structured parsing.
pub fn predict(prompt: &str) -> Result<String> {
    // Local heuristics or templates could be applied here before calling LLM.
    // For now, forward to api_client.
    let ctx = format!("You are Aethr — propose a single, safe shell command for intent: {}\nRespond concisely.", prompt);
    let resp = api_client::call_claude(&ctx)?;
    Ok(resp)
}
