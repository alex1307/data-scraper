use std::collections::HashMap;

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

pub fn build_autouncle_all_searches() -> Vec<HashMap<String, String>> {
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

pub fn build_mobile_bg_all_searches() -> Vec<HashMap<String, String>> {
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

pub fn build_mobile_bg_new_searches() -> Vec<HashMap<String, String>> {
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

pub fn build_cars_bg_new_searches() -> Vec<HashMap<String, String>> {
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

pub fn build_cars_bg_all_searches() -> Vec<HashMap<String, String>> {
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
