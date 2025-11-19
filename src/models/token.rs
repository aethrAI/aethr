// Token data structure

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiToken {
    pub token: String,
    pub created_at: i64,
    pub expires_at: Option<i64>,
}

impl ApiToken {
    pub fn new(token: String) -> Self {
        Self {
            token,
            created_at: chrono::Utc::now().timestamp(),
            expires_at: None,
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            chrono::Utc::now().timestamp() > expires_at
        } else {
            false
        }
    }
}
