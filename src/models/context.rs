// Context data structure

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub id: Option<i64>,
    pub name: String,
    pub detected_at: i64,
    pub metadata: HashMap<String, String>,
}

impl ExecutionContext {
    pub fn new(name: String) -> Self {
        Self {
            id: None,
            name,
            detected_at: chrono::Utc::now().timestamp(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}
