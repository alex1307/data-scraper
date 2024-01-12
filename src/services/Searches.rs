use std::{
    collections::HashMap,
    fs,
    io::{BufWriter, Write},
};

use log::info;

use crate::{scraper::Traits::ScraperTrait, services::ScraperAppService::MOBILE_BG_CRAWLER};

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
            } else {
                result.push(search.clone());
            }
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
    let mut map = HashMap::new();
    map.insert("subm".to_owned(), "1".to_owned());
    map.insert("add_search".to_owned(), "1".to_owned());
    map.insert("typeoffer".to_owned(), "1".to_owned());
    map.insert("last".to_owned(), "1".to_owned());
    map.insert("conditions[]".to_owned(), "1".to_owned());
    map.insert("yearFrom".to_owned(), "2014".to_owned());
    map.insert("company_type[]".to_owned(), "[1,2]".to_owned());
    let id = "2014_now";
    price_filter(id, "priceFrom", "priceTo", map.clone())
}

pub fn cars_bg_all_searches() -> Vec<HashMap<String, String>> {
    let mut map = HashMap::new();
    map.insert("subm".to_owned(), "1".to_owned());
    map.insert("add_search".to_owned(), "1".to_owned());
    map.insert("typeoffer".to_owned(), "1".to_owned());
    map.insert("conditions[]".to_owned(), "1".to_owned());
    map.insert("company_type[]".to_owned(), "[1,2]".to_owned());
    let mut searches = vec![];
    for year in 2014..2024 {
        map.insert("yearFrom".to_owned(), year.to_string());
        map.insert("yearTo".to_owned(), (year + 1).to_string());
        let id = format!("{}_{}", year, year + 1);
        let price_filter = price_filter(&id, "priceFrom", "priceTo", map.clone());
        searches.extend(price_filter);
    }

    searches
}

pub fn mobile_bg_new_searches() -> Vec<HashMap<String, String>> {
    let mut params = HashMap::new();
    params.insert("act".to_owned(), "3".to_owned());
    params.insert("f10".to_owned(), "2014".to_owned());
    params.insert("topmenu".to_string(), "1".to_string());
    params.insert("rub".to_string(), 1.to_string());
    params.insert("pubtype".to_string(), 1.to_string());
    params.insert("f20".to_string(), 7.to_string());
    params.insert("f24".to_string(), 2.to_string());
    let id = "2014_now";
    price_filter(id, "f7", "f8", params.clone())
}

pub fn mobile_bg_all_searches() -> Vec<HashMap<String, String>> {
    let mut params = HashMap::new();
    params.insert("act".to_owned(), "3".to_owned());
    params.insert("topmenu".to_string(), "1".to_string());
    params.insert("rub".to_string(), 1.to_string());
    params.insert("pubtype".to_string(), 1.to_string());
    params.insert("f24".to_string(), 2.to_string());
    let mut meta_searches = vec![];
    for year in 2014..2024 {
        params.insert("f10".to_owned(), year.to_string());
        params.insert("f11".to_owned(), (year + 1).to_string());
        let id = format!("{}_{}", year, year + 1);
        let price_filter = price_filter(&id, "f7", "f8", params.clone());
        meta_searches.extend(price_filter);
    }

    meta_searches
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
                info!("slink: {} for search: {:?}", slink, search);
                searches.push(params.clone());
            }
            Err(e) => {
                info!("Error: {}", e);
                continue;
            }
        }
    }
    searches
}

fn price_filter(
    id: &str,
    price_from: &str,
    price_to: &str,
    source: HashMap<String, String>,
) -> Vec<HashMap<String, String>> {
    let prices = [
        (1000, 2000),
        (2000, 3000),
        (3000, 5000),
        (5000, 7000),
        (7000, 9000),
        (9_000, 11_000),
        (11_000, 13_000),
        (13_000, 15_000),
        (15_000, 20_000),
        (20_000, 25_000),
        (30_000, 40_000),
        (40_000, 50_000),
        (50_000, 90_000),
    ];
    let mut searches = vec![];
    for from_to in prices.iter() {
        let mut params = source.clone();
        params.insert(
            "id".to_owned(),
            format!("{}_{}_{}", id, from_to.0, from_to.1),
        );
        params.insert(price_from.to_owned(), from_to.0.to_string());
        params.insert(price_to.to_owned(), from_to.1.to_string());
        searches.push(params.clone());
    }
    let mut most_expensive = source.clone();
    most_expensive.insert("id".to_owned(), id.to_string());
    most_expensive.insert(price_from.to_owned(), "90000".to_owned());
    searches.push(most_expensive);
    searches
}

pub fn autouncle_all_searches() -> Vec<HashMap<String, String>> {
    //https://www.autouncle.ro/en/cars_search?s%5Bmax_price%5D=5000&s%5Bmin_price%5D=1000&s%5Bmin_year%5D=2004&s%5Bnot_damaged%5D=true
    let mut searches = vec![];
    let mut map = HashMap::new();
    map.insert("s%5Bnot_damaged%5D".to_owned(), "true".to_owned());
    map.insert("s%5Bseller_kind%5D".to_owned(), "Dealer".to_owned());
    map.insert(
        "s%5Bwith_ratings%5D%5B%5D".to_owned(),
        "[1,2,3,4,5]".to_owned(),
    );
    for year in 2014..2024 {
        map.insert("s%5Bmin_year%5D".to_owned(), year.to_string());
        map.insert("s%5Bmax_year%5D".to_owned(), (year + 1).to_string());
        let id = format!("{}_{}", year, year + 1);
        let price_filter = price_filter(&id, "s%5Bmin_price%5D", "s%5Bmax_price%5D", map.clone());
        searches.extend(price_filter);
    }
    searches
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
