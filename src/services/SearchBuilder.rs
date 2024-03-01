use lazy_static::lazy_static;
use log::info;
use std::collections::HashMap;

lazy_static! {
    static ref POWER: Vec<(&'static str, &'static str)> = vec![
        ("75", "90"),
        ("90", "101"),
        ("101", "131"),
        ("131", "150"),
        ("150", "200"),
        ("200", "252"),
        ("252", "303"),
        ("303", "358"),
        ("358", "402"),
        ("402", ""),
    ];
    static ref YEARS: Vec<(&'static str, &'static str)> = vec![
        ("2014", "2015"),
        ("2016", "2017"),
        ("2018", "2019"),
        ("2020", "2021"),
        ("2022", "2024"),
    ];
    static ref CARS_BG_FUELS: Vec<(&'static str, &'static str)> = vec![
        ("[1]", "Petrol"),
        ("[2]", "Diesel"),
        ("[3]", "LPG"),
        ("[6]", "Hybrid"),
        ("[7]", "Electric"),
    ];
    static ref MOBILE_BG_FUELS: Vec<(&'static str, &'static str)> = vec![
        ("benzinov", "Petrol"),
        ("dizelov", "Diesel"),
        ("elektricheski", "Electric"),
        ("hibriden", "Hybrid"),
        ("plug-in-hibrid", "Plug-in-hybrid"),
    ];
    static ref CARS_BG_GEARBOX: Vec<(&'static str, &'static str)> =
        vec![("1", "Manual"), ("2", "Automatic"),];
    static ref MOBILE_BG_GEARBOX: Vec<(&'static str, &'static str)> =
        vec![("rachna", "Manual"), ("avtomatichna", "Automatic"),];
    pub static ref EXCLUED: Vec<&'static str> = vec!["seller", "engine", "gearbox", "power", "id"];
}

const MOBILE_BG_FUEL_ID: &str = "engine_url";
const MOBILE_BG_GEARBOX_ID: &str = "gearbox_url";
pub const MOBILE_BG_POWER_FROM: &str = "powerFrom";
pub const MOBILE_BG_POWER_TO: &str = "powerTo";
pub const MOBILE_BG_YEARS_FROM: &str = "yearFrom";
pub const MOBILE_BG_YEARS_TO: &str = "yearTo";
pub const SE_BG_SELLER_TO: &str = "dealer";

const CARS_BG_FUEL_ID: &str = "fuelId%5B%5D";
pub const CARS_BG_GEARBOX_ID: &str = "gearId";
pub const CARS_BG_YEARS_FROM: &str = "yearFrom";
pub const CARS_BG_YEARS_TO: &str = "yearTo";
pub const CARS_BG_POWER_FROM: &str = "powerFrom";
pub const CARS_BG_POWER_TO: &str = "powerTo";
// const CARS_BG_PRICE_FROM: &str = "priceFrom";
// const CARS_BG_PRICE_TO: &str = "priceTo";

pub const CRAWLER_KEY: &str = "crawler_key";
pub const CRAWLER_MOBILE_BG: &str = "mobile.bg";
pub const CRAWLER_CARS_BG: &str = "cars.bg";
pub const CRAWLER_AUTOUNCLE_RO: &str = "autouncle.ro";
pub const CRAWLER_AUTOUNCLE_NL: &str = "autouncle.nl";
pub const CRAWLER_AUTOUNCLE_FR: &str = "autouncle.fr";

fn fuel_filter(
    fuelid: &str,
    source: Vec<(&'static str, &'static str)>,
) -> Vec<HashMap<String, String>> {
    let mut searches = vec![];
    let mut params = HashMap::new();
    for fuel in source.iter() {
        params.insert(fuelid.to_owned(), fuel.0.to_string());
        params.insert("engine".to_owned(), fuel.1.to_string());
        searches.push(params.clone());
    }
    searches
}

fn gear_box_filter(
    gear_id: &str,
    source: Vec<(&'static str, &'static str)>,
) -> Vec<HashMap<String, String>> {
    let mut searches = vec![];
    let mut params = HashMap::new();
    for gearbox in source.iter() {
        params.insert(gear_id.to_owned(), gearbox.0.to_string());
        params.insert("gearbox".to_owned(), gearbox.1.to_string());
        searches.push(params.clone());
    }
    searches
}
fn power_filter(
    power_from: &str,
    power_to: &str,
    source: Vec<(&'static str, &'static str)>,
) -> Vec<HashMap<String, String>> {
    let mut searches = vec![];
    for power in source.iter() {
        let mut params = HashMap::new();
        if !power.1.is_empty() {
            params.insert(power_to.to_owned(), power.1.to_string());
            params.insert("power".to_owned(), power.1.to_string());
        } else {
            params.insert("power".to_owned(), "9999".to_string());
        }
        if !power.0.is_empty() {
            params.insert(power_from.to_owned(), power.0.to_string());
        }

        searches.push(params);
    }
    searches
}

fn year_filter(
    year_from: &str,
    year_to: &str,
    source: Vec<(&'static str, &'static str)>,
) -> Vec<HashMap<String, String>> {
    let mut searches = vec![];
    for year in source.iter() {
        let mut params = HashMap::new();
        if !year.1.is_empty() {
            params.insert(year_to.to_owned(), year.1.to_string());
        }
        if !year.0.is_empty() {
            params.insert(year_from.to_owned(), year.0.to_string());
        }
        searches.push(params);
    }
    searches
}

fn price_filter(
    price_from: &str,
    price_to: &str,
    source: HashMap<String, String>,
) -> Vec<HashMap<String, String>> {
    let prices = [
        (1000, 5000),
        (5000, 10_000),
        (10_000, 13_000),
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
        params.insert(price_from.to_owned(), from_to.0.to_string());
        params.insert(price_to.to_owned(), from_to.1.to_string());
        searches.push(params.clone());
    }
    let mut most_expensive = source.clone();
    most_expensive.insert(price_from.to_owned(), "90000".to_owned());
    searches.push(most_expensive);
    searches
}

pub fn build_autouncle_ro_searches() -> Vec<HashMap<String, String>> {
    //https://www.autouncle.ro/en/cars_search?s%5Bmax_price%5D=5000&s%5Bmin_price%5D=1000&s%5Bmin_year%5D=2004&s%5Bnot_damaged%5D=true
    let mut searches = vec![];
    let mut map = HashMap::new();
    map.insert("s%5Bnot_damaged%5D".to_owned(), "true".to_owned());
    map.insert("s%5Bseller_kind%5D".to_owned(), "Dealer".to_owned());
    map.insert(
        "s%5Bwith_ratings%5D%5B%5D".to_owned(),
        "[5,4,3,2,1]".to_owned(),
    );
    let year_filter = year_filter("s%5Bmin_year%5D", "s%5Bmax_year%5D", YEARS.clone());
    let price_filter = price_filter("s%5Bmin_price%5D", "s%5Bmax_price%5D", map.clone());

    for year in year_filter {
        for price in price_filter.iter() {
            let mut params = map.clone();
            params.extend(year.clone());
            params.extend(price.clone());
            params.insert(CRAWLER_KEY.to_owned(), CRAWLER_AUTOUNCLE_RO.to_owned());
            searches.push(params);
        }
    }
    searches
}
pub fn build_autouncle_nl_searches() -> Vec<HashMap<String, String>> {
    //https://www.autouncle.nl/en/cars_search?s%5Bmax_price%5D=5000&s%5Bmin_price%5D=1000&s%5Bmin_year%5D=2004&s%5Bnot_damaged%5D=true
    let mut searches = vec![];
    let mut map = HashMap::new();
    map.insert("s%5Bnot_damaged%5D".to_owned(), "true".to_owned());
    map.insert("s%5Bseller_kind%5D".to_owned(), "Dealer".to_owned());
    map.insert("s%5Bwith_ratings%5D%5B%5D".to_owned(), "[5]".to_owned());
    map.insert("s%5Bfeatured%5D".to_owned(), "true".to_owned());
    let year_filter = year_filter("s%5Bmin_year%5D", "s%5Bmax_year%5D", YEARS.clone());
    let price_filter = price_filter("s%5Bmin_price%5D", "s%5Bmax_price%5D", map.clone());

    for year in year_filter {
        for price in price_filter.iter() {
            let mut params = map.clone();
            params.extend(year.clone());
            params.extend(price.clone());
            params.insert(CRAWLER_KEY.to_owned(), CRAWLER_AUTOUNCLE_NL.to_owned());
            searches.push(params);
        }
    }
    searches
}

pub fn build_autouncle_fr_searches() -> Vec<HashMap<String, String>> {
    //https://www.autouncle.fr/en/cars_search?s%5Bmax_price%5D=5000&s%5Bmin_price%5D=1000&s%5Bmin_year%5D=2004&s%5Bnot_damaged%5D=true
    let mut searches = vec![];
    let mut map = HashMap::new();
    map.insert("s%5Bseller_kind%5D".to_owned(), "Dealer".to_owned());
    map.insert("s%5Bwith_ratings%5D%5B%5D".to_owned(), "[5]".to_owned());
    map.insert("s%5Bfeatured%5D".to_owned(), "true".to_owned());
    let year_filter = year_filter("s%5Bmin_year%5D", "s%5Bmax_year%5D", YEARS.clone());
    let price_filter = price_filter("s%5Bmin_price%5D", "s%5Bmax_price%5D", map.clone());

    for year in year_filter {
        for price in price_filter.iter() {
            let mut params = map.clone();
            params.extend(year.clone());
            params.extend(price.clone());
            params.insert(CRAWLER_KEY.to_owned(), CRAWLER_AUTOUNCLE_FR.to_owned());
            searches.push(params);
        }
    }
    info!("Search builder: searches: {}", searches.len());
    searches
}

pub fn build_mobile_bg_all_searches() -> Vec<HashMap<String, String>> {
    info!("Building mobile.bg all searches");
    let base = HashMap::from([("f24".to_owned(), "2".to_owned())]);
    let mut searches = vec![];
    let year_filter = year_filter(MOBILE_BG_YEARS_FROM, MOBILE_BG_YEARS_TO, YEARS.clone());
    let power_filter = power_filter(MOBILE_BG_POWER_FROM, MOBILE_BG_POWER_TO, POWER.clone());
    let fuel_filter = fuel_filter(MOBILE_BG_FUEL_ID, MOBILE_BG_FUELS.clone());
    let gearbox_filter = gear_box_filter(MOBILE_BG_GEARBOX_ID, MOBILE_BG_GEARBOX.clone());

    for fuel in fuel_filter.iter() {
        for gearbox in gearbox_filter.iter() {
            for power in power_filter.iter() {
                for year in year_filter.iter() {
                    let mut params = base.clone();
                    params.extend(fuel.clone());
                    params.extend(gearbox.clone());
                    params.extend(power.clone());
                    params.extend(year.clone());
                    params.insert(CRAWLER_KEY.to_owned(), CRAWLER_MOBILE_BG.to_owned());
                    searches.push(params);
                }
            }
        }
    }
    info!("Search builder: searches: {}", searches.len());
    searches
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
    price_filter("f7", "f8", params.clone())
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
    price_filter("priceFrom", "priceTo", map.clone())
}

pub fn build_cars_bg_all_searches() -> Vec<HashMap<String, String>> {
    let mut map = HashMap::new();
    map.insert("subm".to_owned(), "1".to_owned());
    map.insert("add_search".to_owned(), "1".to_owned());
    map.insert("typeoffer".to_owned(), "1".to_owned());
    map.insert("conditions[]".to_owned(), "1".to_owned());

    let mut searches = vec![];
    let year_filter = year_filter(CARS_BG_YEARS_FROM, CARS_BG_YEARS_TO, YEARS.clone());
    let power_filter = power_filter(CARS_BG_POWER_FROM, CARS_BG_POWER_TO, POWER.clone());
    let fuel_filter = fuel_filter(CARS_BG_FUEL_ID, CARS_BG_FUELS.clone());
    let gearbox_filter = gear_box_filter(CARS_BG_GEARBOX_ID, CARS_BG_GEARBOX.clone());
    for fuel in fuel_filter.iter() {
        for gearbox in gearbox_filter.iter() {
            for power in power_filter.iter() {
                for year in year_filter.iter() {
                    let mut params = map.clone();
                    params.extend(fuel.clone());
                    params.extend(gearbox.clone());
                    params.extend(power.clone());
                    params.extend(year.clone());
                    params.insert(CRAWLER_KEY.to_owned(), CRAWLER_CARS_BG.to_owned());
                    searches.push(params);
                }
            }
        }
    }
    searches
}
