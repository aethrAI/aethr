use crate::Result;
use regex::Regex;
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct RulesFile {
    rules: Vec<RuleDef>,
}

#[derive(Debug, Deserialize)]
struct RuleDef {
    name: String,
    match_regex: String,
    fix_command: String,
    confidence: Option<f32>,
    explanation: Option<String>,
}

/// Load rules YAML (path) and attempt to match the error_text.
/// Returns Some((command, confidence, explanation)) when matched.
pub fn apply_rules_from_path<P: AsRef<Path>>(path: P, error_text: &str) -> Result<Option<(String, f32, String)>> {
    if !path.as_ref().exists() {
        return Ok(None);
    }
    let yaml = fs::read_to_string(path)?;
    let parsed: RulesFile = serde_yaml::from_str(&yaml)?;
    for r in parsed.rules {
        if let Ok(re) = Regex::new(&r.match_regex) {
            if let Some(caps) = re.captures(error_text) {
                // Replace named capture groups into fix_command if present
                let mut fix = r.fix_command.clone();
                // support named groups like (?P<module>...) and $1
                for name in re.capture_names().flatten() {
                    if let Some(m) = caps.name(name) {
                        fix = fix.replace(&format!("{{{{{}}}}}", name), m.as_str());
                    }
                }
                // If numeric groups used ($1, $2) handle a few
                for i in 1..=5 {
                    if let Some(m) = caps.get(i) {
                        fix = fix.replace(&format!("${}", i), m.as_str());
                    }
                }
                let conf = r.confidence.unwrap_or(0.6);
                let expl = r.explanation.unwrap_or_else(|| "".into());
                return Ok(Some((fix, conf, expl)));
            }
        }
    }
    Ok(None)
}