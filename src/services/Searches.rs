use std::collections::HashMap;

use log::info;

use crate::{scraper::Traits::ScraperTrait, services::ScraperAppService::MOBILE_BG_CRAWLER};

pub fn cars_bg_new_searches() -> Vec<HashMap<String, String>> {
    let mut map = HashMap::new();
    map.insert("subm".to_owned(), "1".to_owned());
    map.insert("add_search".to_owned(), "1".to_owned());
    map.insert("typeoffer".to_owned(), "1".to_owned());
    map.insert("last".to_owned(), "1".to_owned());
    map.insert("conditions[]".to_owned(), "1".to_owned());
    map.insert("yearFrom".to_owned(), "2004".to_owned());
    price_filter("priceFrom", "priceTo", map.clone())
}

pub fn cars_bg_all_searches() -> Vec<HashMap<String, String>> {
    let mut map = HashMap::new();
    map.insert("subm".to_owned(), "1".to_owned());
    map.insert("add_search".to_owned(), "1".to_owned());
    map.insert("typeoffer".to_owned(), "1".to_owned());
    map.insert("conditions[]".to_owned(), "1".to_owned());
    let mut searches = vec![];
    for year in 2004..2023 {
        map.insert("yearFrom".to_owned(), year.to_string());
        map.insert("yearTo".to_owned(), (year + 1).to_string());
        let price_filter = price_filter("priceFrom", "priceTo", map.clone());
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
        let html = MOBILE_BG_CRAWLER.get_html(search.clone(), 1).await.unwrap();
        let slink = MOBILE_BG_CRAWLER.slink(&html).unwrap();
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

pub fn car_gr_new_searches() -> Vec<HashMap<String, String>> {
    let mut map = HashMap::new();
    map.insert("category".to_owned(), "15001".to_owned());
    map.insert("media_types".to_owned(), "photo".to_owned());
    map.insert("withprice".to_owned(), "1".to_owned());
    map.insert("created".to_owned(), ">1".to_owned());
    map.insert("lang".to_owned(), "en".to_owned());
    map.insert("registration-from".to_owned(), "2004".to_owned());
    // map.insert("registration-to".to_owned(), "2008".to_owned());
    let price_filter = price_filter("price-from", "price-to", map.clone());
    price_filter
}

pub fn car_gr_all_searches() -> Vec<HashMap<String, String>> {
    let mut searches = vec![];
    let mut map = HashMap::new();
    map.insert("withprice".to_owned(), "1".to_owned());
    for year in 2022..2023 {
        map.insert("registration-from".to_owned(), year.to_string());
        map.insert("registration-to".to_owned(), (year + 1).to_string());
        let price_filter = price_filter("price-from", "price-to", map.clone());
        searches.extend(price_filter);
    }
    searches
}

pub fn autouncle_all_searches() -> Vec<HashMap<String, String>> {
    //https://www.autouncle.ro/en/cars_search?s%5Bmax_price%5D=5000&s%5Bmin_price%5D=1000&s%5Bmin_year%5D=2004&s%5Bnot_damaged%5D=true
    let mut searches = vec![];
    let mut map = HashMap::new();
    map.insert("s%5Bnot_damaged%5D".to_owned(), "true".to_owned());
    for year in 2003..2023 {
        map.insert("s%5Bmin_year%5D".to_owned(), year.to_string());
        map.insert("s%5Bmax_year%5D".to_owned(), (year + 1).to_string());
        let price_filter = price_filter("s%5Bmin_price%5D", "s%5Bmax_price%5D", map.clone());
        searches.extend(price_filter);
    }
    searches
}
