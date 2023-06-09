pub mod config;
pub mod scraper;
pub mod model;
pub mod services;
pub mod utils;
pub mod writer;

use lazy_static::lazy_static;

use std::sync::Once;

pub const LISTING_URL: &str = "//www.mobile.bg/pcgi/mobile.cgi?act=3";
pub const DETAILS_URL: &str = "//www.mobile.bg/pcgi/mobile.cgi?act=4";
pub const ACTION_DETAILS: &str = "act=4";
pub const ACTION_LIST: &str = "act=3";
pub const DATE_FORMAT: &str = "%Y-%m-%d";
pub const DATE_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
pub const NOT_FOUND_MSG: &str = "изтрита или не е активна";
pub const POWER_TXT: &str = "Мощност";
pub const GEARBOX_TXT: &str = "Скоростна кутия";
pub const ENGINE_TXT: &str = "Тип двигател";
pub const BROWSER_USER_AGENT: &str ="Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.1 Safari/605.1.15";

lazy_static! {
    static ref INIT_LOGGER: Once = Once::new();
}
