use std::collections::HashMap;

use super::SearchBuilder::{build_cars_bg_all_searches, ID_CARS_BG_START};

pub const MOBILE_BG_NEW_SEARCHES: &str = "resources/searches/mobile_bg_new_search.json";
pub const MOBILE_BG_ALL_SEARCHES: &str = "resources/searches/mobile_bg_all_search.json";

pub const CARS_BG_NEW_SEARCHES: &str = "resources/searches/cars_bg_new_search.json";
pub const CARS_BG_ALL_SEARCHES: &str = "resources/searches/cars_bg_all_search.json";

pub const AUTOUNCLE_ALL_SEARCHES: &str = "resources/searches/autouncle_all_search.json";

pub fn cars_bg_all_searches() -> Vec<HashMap<String, String>> {
    build_cars_bg_all_searches("https://www.cars.bg/carslist.php?", ID_CARS_BG_START)
}
