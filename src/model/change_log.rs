use serde::{Deserialize, Serialize};

use super::traits::{Header, Identity};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ChangeLog {
    pub timestamp: i64,
    pub id: String,
    pub status: String,
    pub created_on: String,
}

impl Identity for ChangeLog {
    fn get_id(&self) -> String {
        self.id.clone()
    }
}

impl Header for ChangeLog {
    fn header() -> Vec<&'static str> {
        vec!["timestamp", "id", "status", "created_on"]
    }
}
