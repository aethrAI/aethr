// Command data structure

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub id: Option<i64>,
    pub command: String,
    pub executed_at: i64,
    pub exit_code: Option<i32>,
    pub working_directory: String,
    pub context_id: Option<i64>,
}

impl Command {
    pub fn new(command: String, working_directory: String) -> Self {
        Self {
            id: None,
            command,
            executed_at: chrono::Utc::now().timestamp(),
            exit_code: None,
            working_directory,
            context_id: None,
        }
    }

    pub fn with_exit_code(mut self, exit_code: i32) -> Self {
        self.exit_code = Some(exit_code);
        self
    }
}
