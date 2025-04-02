use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct JsonSearchSet(pub Vec<HashMap<String, String>>);

pub struct JsonSearchService;

impl JsonSearchService {
    pub fn load_searches_from_dir<P: AsRef<Path>>(
        dir: P,
    ) -> Result<Vec<HashMap<String, String>>, String> {
        let mut all_searches = Vec::new();

        let entries = fs::read_dir(dir).map_err(|e| format!("Failed to read directory: {}", e))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();
            if path.extension().map(|s| s == "json").unwrap_or(false) {
                let content = fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read file {:?}: {}", path, e))?;
                let searches: Vec<HashMap<String, String>> = serde_json::from_str(&content)
                    .map_err(|e| format!("Failed to parse JSON from {:?}: {}", path, e))?;
                all_searches.extend(searches);
            }
        }

        Ok(all_searches)
    }
}
