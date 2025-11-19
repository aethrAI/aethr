use crate::Result;
use crate::utils::api_client;
use serde::Deserialize;

/// Structured suggestion returned by `fix_error`.
#[derive(Debug, Deserialize)]
pub struct FixSuggestion {
    pub command: Option<String>,
    pub confidence: Option<f32>,
    pub explanation: Option<String>,
}

/// Ask LLM for a fix for `error_text`. Attempts to parse JSON response into FixSuggestion.
/// If parsing fails, the raw model text is returned in FixSuggestion.explanation.
pub fn fix_error(error_text: &str) -> Result<FixSuggestion> {
    let prompt = format!(
        "You are Aethr. Given this terminal error:\n{}\nReturn a JSON object: {{\"command\":\"...\",\"confidence\":0.0,\"explanation\":\"...\"}}. If uncertain, set command to an empty string and confidence < 0.6.",
        error_text
    );

    let resp = api_client::call_claude(&prompt)?;

    // Try to parse JSON from the response
    if let Ok(parsed) = serde_json::from_str::<FixSuggestion>(&resp) {
        return Ok(parsed);
    }

    // If response isn't strict JSON, attempt to find a JSON substring
    if let Some(start) = resp.find('{') {
        if let Some(end) = resp.rfind('}') {
            if end > start {
                if let Ok(parsed) = serde_json::from_str::<FixSuggestion>(&resp[start..=end]) {
                    return Ok(parsed);
                }
            }
        }
    }

    // Fallback: return raw output as explanation
    Ok(FixSuggestion {
        command: None,
        confidence: None,
        explanation: Some(resp),
    })
}
