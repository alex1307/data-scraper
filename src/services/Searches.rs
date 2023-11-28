use std::collections::HashMap;

use log::info;

use crate::services::ScraperAppService::MOBILE_BG_CRAWLER;

pub fn cars_bg_new_searches() -> Vec<HashMap<String, String>> {
    let mut map = HashMap::new();
    map.insert("subm".to_owned(), "1".to_owned());
    map.insert("add_search".to_owned(), "1".to_owned());
    map.insert("typeoffer".to_owned(), "1".to_owned());
    map.insert("last".to_owned(), "1".to_owned());
    map.insert("conditions[]".to_owned(), "1".to_owned());
    map.insert("yearFrom".to_owned(), "2004".to_owned());
    map.insert("steering_wheel".to_owned(), "1".to_owned());
    price_filter("priceForm", "priceTo", map.clone())
}

pub fn cars_bg_all_searches() -> Vec<HashMap<String, String>> {
    let mut map = HashMap::new();
    map.insert("subm".to_owned(), "1".to_owned());
    map.insert("add_search".to_owned(), "1".to_owned());
    map.insert("typeoffer".to_owned(), "1".to_owned());
    map.insert("last".to_owned(), "1".to_owned());
    map.insert("conditions[]".to_owned(), "1".to_owned());
    map.insert("steering_wheel".to_owned(), "1".to_owned());
    let mut searches = vec![];
    for year in 2004..2023 {
        map.insert("yearFrom".to_owned(), year.to_string());
        map.insert("yearTo".to_owned(), (year + 1).to_string());
        let price_filter = price_filter("priceForm", "priceTo", map.clone());
        searches.extend(price_filter);
    }
    searches
}

pub async fn mobile_bg_new_searches() -> Vec<HashMap<String, String>> {
    let mut searches = vec![];
    let mut params = HashMap::new();
    params.insert("act".to_owned(), "3".to_owned());
    params.insert("f10".to_owned(), "2004".to_owned());
    params.insert("topmenu".to_string(), "1".to_string());
    params.insert("rub".to_string(), 1.to_string());
    params.insert("pubtype".to_string(), 1.to_string());
    params.insert("f20".to_string(), 7.to_string());
    let mut meta_searches = price_filter("f7", "f8", params.clone());

    let mut sold_vehicles = params.clone();
    sold_vehicles.insert(
        "f94".to_string(),
        "1~%CA%E0%EF%E0%F0%E8%F0%E0%ED%5C%CF%F0%EE%E4%E0%E4%E5%ED".to_string(),
    );
    meta_searches.push(sold_vehicles);

    params.clear();
    params.insert("act".to_owned(), "3".to_owned());
    params.insert("rub".to_string(), 1.to_string());
    params.insert("pubtype".to_string(), 1.to_string());
    params.insert("topmenu".to_string(), "1".to_string());

    for search in meta_searches.clone() {
        let slink = MOBILE_BG_CRAWLER.slink(search.clone()).await.unwrap();
        params.insert("slink".to_owned(), slink.clone());
        info!("slink: {} for search: {:?}", slink, search);
        searches.push(params.clone());
    }
    searches
}

pub async fn mobile_bg_all_searches() -> Vec<HashMap<String, String>> {
    let mut searches = vec![];
    let mut params = HashMap::new();
    params.insert("act".to_owned(), "3".to_owned());
    params.insert("topmenu".to_string(), "1".to_string());
    params.insert("rub".to_string(), 1.to_string());
    params.insert("pubtype".to_string(), 1.to_string());
    let mut meta_searches = vec![];
    for year in 2004..2023 {
        params.insert("f10".to_owned(), year.to_string());
        params.insert("f11".to_owned(), (year + 1).to_string());
        let price_filter = price_filter("f7", "f8", params.clone());
        meta_searches.extend(price_filter);
    }

    params.clear();
    params.insert("act".to_owned(), "3".to_owned());
    params.insert("rub".to_string(), 1.to_string());
    params.insert("pubtype".to_string(), 1.to_string());
    params.insert("topmenu".to_string(), "1".to_string());

    for search in meta_searches.clone() {
        let slink = MOBILE_BG_CRAWLER.slink(search.clone()).await.unwrap();
        params.insert("slink".to_owned(), slink.clone());
        info!("slink: {} for search: {:?}", slink, search);
        searches.push(params.clone());
    }
    searches
}

fn price_filter(
    price_from: &str,
    price_to: &str,
    source: HashMap<String, String>,
) -> Vec<HashMap<String, String>> {
    let prices = [
        1000, 2000, 3000, 4000, 5000, 6000, 7000, 8000, 9000, 10_000, 11_000, 13_000, 15_000,
        18_000, 19_000, 21_000, 25_000, 30_000, 40_000, 50_000, 95_000,
    ];
    let mut searches = vec![];
    for i in 0..prices.len() - 2 {
        let mut params = source.clone();
        params.insert(price_from.to_owned(), prices[i].to_string());
        params.insert(price_to.to_owned(), (prices[i + 1]).to_string());
        searches.push(params.clone());
    }
    let mut most_expensive = source.clone();
    most_expensive.insert(price_from.to_owned(), "95000".to_owned());
    searches.push(most_expensive);
    searches
}
