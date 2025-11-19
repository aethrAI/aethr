use crate::Result;
use crate::db::local::LocalDB;
use crate::utils::config;
use serde::Deserialize;

/// Try local-first recall. If local DB has results, return them.
/// If no local results or DB missing, optionally forward to daemon (if available).
pub fn recall_local_first(query: &str, limit: usize) -> Result<Vec<String>> {
    // Try local DB first
    let db_path = config::get_db_path();
    if db_path.exists() {
        let db = LocalDB::new(&db_path)?;
        let results = db.search(query, limit)?;
        if !results.is_empty() {
            return Ok(results);
        }
    }

    // If local missed, try talking to daemon via Unix socket (best-effort)
    #[cfg(unix)]
    {
        use crate::daemon::client;
        use serde_json::json;

        let payload = json!({
            "method": "recall",
            "params": {
                "query": query,
                "limit": limit
            }
        })
        .to_string();

        if let Ok(resp) = client::send_request(&payload) {
            // Expect daemon to return JSON like: {"items": ["cmd1","cmd2"...]}
            if let Ok(parsed) = serde_json::from_str::<DaemonRecallResponse>(&resp) {
                return Ok(parsed.items);
            } else {
                // Fallback: treat response as a single-line command
                if !resp.trim().is_empty() {
                    return Ok(vec![resp.trim().to_string()]);
                }
            }
        }
    }

    // Nothing found
    Ok(Vec::new())
}

#[derive(Deserialize)]
struct DaemonRecallResponse {
    items: Vec<String>,
}
