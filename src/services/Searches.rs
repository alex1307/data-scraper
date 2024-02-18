use std::{
    collections::HashMap,
    fs,
    io::{BufWriter, Write},
};

use log::info;

use crate::{
    scraper::Traits::ScraperTrait, services::ScraperAppService::MOBILE_BG_CRAWLER,
    AUTOUNCLE_ALL_SEARCHES_LOG, CARS_BG_NEW_SEARCHES_LOG, MOBILE_BG_NEW_SEARCHES_LOG,
};

use super::SearchBuilder::{build_cars_bg_all_searches, build_mobile_bg_all_searches};

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

pub fn save_searches(file_name: &str, searches: Vec<HashMap<String, String>>) {
    if searches.is_empty() {
        return;
    }
    let content = fs::read_to_string(file_name).unwrap();
    let mut source = serde_json::from_str::<Vec<HashMap<String, String>>>(&content).unwrap();
    source.extend(searches);
    let json_data = serde_json::to_string_pretty(&source).unwrap();
    fs::write(file_name, json_data).unwrap();
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

pub async fn to_slink(search: &mut HashMap<String, String>) {
    let html = MOBILE_BG_CRAWLER.get_html(search.clone(), 1).await.unwrap();
    match MOBILE_BG_CRAWLER.slink(&html) {
        Ok(slink) => {
            info!("slink: {}", slink);
            search.insert("slink".to_owned(), slink);
        }
        Err(e) => {
            info!("Error: {}", e);
        }
    }
}

pub async fn to_slink_searches(
    meta_searches: Vec<HashMap<String, String>>,
) -> Vec<HashMap<String, String>> {
    let mut searches = vec![];
    let mut params = HashMap::new();
    params.insert("act".to_owned(), "3".to_owned());
    params.insert("rub".to_string(), 1.to_string());
    params.insert("pubtype".to_string(), 1.to_string());
    params.insert("topmenu".to_string(), "1".to_string());

    for search in meta_searches {
        let html = MOBILE_BG_CRAWLER.get_html(search.clone(), 1).await.unwrap();
        match MOBILE_BG_CRAWLER.slink(&html) {
            Ok(slink) => {
                params.insert("slink".to_owned(), slink.clone());
                //params.insert("id".to_owned(), search.get("id").unwrap().to_owned());
                searches.push(params.clone());
            }
            Err(e) => {
                info!("Error: {}", e);
                continue;
            }
        }
    }
    info!("Total searches with slink: {}", searches.len());
    searches
}

pub fn autouncle_all_searches() -> Vec<HashMap<String, String>> {
    load_searches(AUTOUNCLE_ALL_SEARCHES, &AUTOUNCLE_ALL_SEARCHES_LOG)
}
#[cfg(test)]
mod test_searches {
    use std::fs;

    use log::info;

    use crate::{
        services::Searches::{cars_bg_new_searches, save_searches},
        utils::helpers::configure_log4rs,
        LOG_CONFIG,
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
        save_searches(MOBILE_BG_NEW_SEARCHES_LOG2, processed);
        let searches = super::load_searches(MOBILE_BG_NEW_SEARCHES, MOBILE_BG_NEW_SEARCHES_LOG2);
        assert_eq!(searches.len(), 10);
        fs::write(MOBILE_BG_NEW_SEARCHES_LOG2, "[]").unwrap();
    }
}
