use std::collections::HashMap;

use chrono::Local;
use serde::{Deserialize, Serialize};

use crate::DATE_FORMAT;

use super::traits::{Header, Identity};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DataError {
    pub id: String,
    pub error: String,
    pub created_on: String,
}

impl From<HashMap<String, String>> for DataError {
    fn from(map: HashMap<String, String>) -> Self {
        DataError {
            id: map.get("id").unwrap().to_string(),
            error: map.get("error").unwrap().to_string(),
            created_on: Local::now().format(DATE_FORMAT).to_string(),
        }
    }
}

impl Header for DataError {
    fn header() -> Vec<&'static str> {
        vec!["id", "error", "created_on"]
    }
}

impl Identity for DataError {
    fn get_id(&self) -> String {
        self.id.clone()
    }
}
