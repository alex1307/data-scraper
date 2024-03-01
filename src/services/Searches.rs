use std::{
    collections::HashMap,
    fs,
    io::{BufWriter, Write},
};

use log::info;

use crate::{CARS_BG_NEW_SEARCHES_LOG, MOBILE_BG_NEW_SEARCHES_LOG};

use super::SearchBuilder::{
    build_autouncle_ro_searches, build_cars_bg_all_searches, build_mobile_bg_all_searches,
};

pub const MOBILE_BG_NEW_SEARCHES: &str = "resources/searches/mobile_bg_new_search.json";
pub const MOBILE_BG_ALL_SEARCHES: &str = "resources/searches/mobile_bg_all_search.json";

pub const CARS_BG_NEW_SEARCHES: &str = "resources/searches/cars_bg_new_search.json";
pub const CARS_BG_ALL_SEARCHES: &str = "resources/searches/cars_bg_all_search.json";

pub const AUTOUNCLE_ALL_SEARCHES: &str = "resources/searches/autouncle_all_search.json";

pub fn init_searches() -> Result<(), String> {
    let searches = cars_bg_new_searches();
    let json_data = serde_json::to_string_pretty(&searches).map_err(|e| e.to_string())?;
    fs::write(CARS_BG_NEW_SEARCHES, json_data).unwrap();

    let searches = cars_bg_all_searches();
    let json_data = serde_json::to_string_pretty(&searches).map_err(|e| e.to_string())?;
    fs::write(CARS_BG_ALL_SEARCHES, json_data).map_err(|e| e.to_string())?;

    let searches = mobile_bg_new_searches();
    let json_data = serde_json::to_string_pretty(&searches).map_err(|e| e.to_string())?;
    fs::write(MOBILE_BG_NEW_SEARCHES, json_data).map_err(|e| e.to_string())?;

    let searches = mobile_bg_all_searches();
    let json_data = serde_json::to_string_pretty(&searches).map_err(|e| e.to_string())?;
    fs::write(MOBILE_BG_ALL_SEARCHES, json_data).map_err(|e| e.to_string())?;

    let searches = autouncle_all_searches();
    let json_data = serde_json::to_string_pretty(&searches).map_err(|e| e.to_string())?;
    fs::write(AUTOUNCLE_ALL_SEARCHES, json_data).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_searches(file_name: &str, log_file_name: &str) -> Vec<HashMap<String, String>> {
    let source = fs::read_to_string(file_name).unwrap();
    let searches: Vec<HashMap<String, String>> = serde_json::from_str(&source).unwrap();
    let path = std::path::Path::new(log_file_name);
    if !path.exists() {
        let file = fs::File::create(path).unwrap();
        let mut writer = BufWriter::new(file);
        writer.write_all("[]".as_bytes()).unwrap();
        return searches;
    }
    let target = fs::read_to_string(log_file_name).unwrap();
    let filtered: Vec<HashMap<String, String>> = serde_json::from_str(&target).unwrap();
    let mut result = vec![];
    for search in searches {
        if let Some(id) = search.get("id") {
            if filtered
                .iter()
                .any(|x| x.get("id").is_some() && x.get("id").unwrap() == id)
            {
                continue;
            }
            result.push(search.clone());
        }
    }
    info!("Total searches: {}", result.len());
    result
}

pub fn cars_bg_new_searches() -> Vec<HashMap<String, String>> {
    load_searches(CARS_BG_NEW_SEARCHES, &CARS_BG_NEW_SEARCHES_LOG)
}

pub fn cars_bg_all_searches() -> Vec<HashMap<String, String>> {
    build_cars_bg_all_searches()
}

pub fn mobile_bg_new_searches() -> Vec<HashMap<String, String>> {
    load_searches(MOBILE_BG_NEW_SEARCHES, &MOBILE_BG_NEW_SEARCHES_LOG)
}

pub fn mobile_bg_all_searches() -> Vec<HashMap<String, String>> {
    build_mobile_bg_all_searches()
}

pub fn autouncle_all_searches() -> Vec<HashMap<String, String>> {
    build_autouncle_ro_searches()
}
#[cfg(test)]
mod test_searches {
    use std::fs;

    use log::info;

    use crate::{
        services::Searches::cars_bg_new_searches, utils::helpers::configure_log4rs, LOG_CONFIG,
    };

    const MOBILE_BG_NEW_SEARCHES: &str = "resources/test-data/searches/mobile_bg_new_search.json";
    const MOBILE_BG_NEW_SEARCHES_LOG: &str =
        "resources/test-data/searches/mobile_bg_new_search-2024-01-12.json";
    const EMPTY_SEARCHES_LOG: &str = "resources/test-data/searches/empty_search.json";
    const MOBILE_BG_NEW_SEARCHES_LOG2: &str =
        "resources/test-data/searches/mobile_bg_new_search-2024-01-13.json";
    #[test]
    fn test_load_searches() {
        let searches = cars_bg_new_searches();
        let json_data = serde_json::to_string_pretty(&searches).unwrap();
        fs::write(MOBILE_BG_NEW_SEARCHES, json_data).unwrap();
        let searches = super::load_searches(MOBILE_BG_NEW_SEARCHES, MOBILE_BG_NEW_SEARCHES_LOG);
        assert_eq!(searches.len(), 14);
    }

    #[test]
    fn test_add_searches() {
        configure_log4rs(&LOG_CONFIG);
        let searches = super::load_searches(MOBILE_BG_NEW_SEARCHES, EMPTY_SEARCHES_LOG);
        let processed = searches[1..5].to_vec();
        info!("processed: {:?}", processed.len());
        let searches = super::load_searches(MOBILE_BG_NEW_SEARCHES, MOBILE_BG_NEW_SEARCHES_LOG2);
        assert_eq!(searches.len(), 10);
        fs::write(MOBILE_BG_NEW_SEARCHES_LOG2, "[]").unwrap();
    }
}
