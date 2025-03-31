#![allow(non_snake_case)]
pub mod config;
pub mod constants;
pub mod helpers;
pub mod kafka;
pub mod model;
pub mod protos;
pub mod scraper;
pub mod services;
pub mod utils;
pub mod writer;
#[macro_use]
mod macros;

use chrono::{DateTime, Utc};
use lazy_static::lazy_static;

use std::sync::Once;

use crate::config::app_config::AppConfig;

pub const CARS_BG_LISTING_URL: &str = r#"https://www.cars.bg/carslist.php?"#;
pub const CARS_BG_DETAILS_URL: &str = r#"https://www.cars.bg/offer"#;

pub const LISTING_URL: &str = "https://www.mobile.bg/pcgi/mobile.cgi?act=3&f10=2004&";
pub const DETAILS_URL: &str = "https://www.mobile.bg/pcgi/mobile.cgi?act=4&";

pub const ACTION_DETAILS: &str = "act=4";
pub const ACTION_LIST: &str = "act=3";
pub const DATE_FORMAT: &str = "%Y-%m-%d";
pub const DATE_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
pub const NOT_FOUND_MSG: &str = "изтрита или не е активна";
pub const POWER_TXT: &str = "Мощност";
pub const GEARBOX_TXT: &str = "Скоростна кутия";
pub const ENGINE_TXT: &str = "Тип двигател";
pub const MILLAGE_TXT: &str = "Пробег";
pub const YEAR_TXT: &str = "Дата на производство";

pub const BROWSER_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.1 Safari/605.1.15";

pub const SPECIAL_MAKES: [&str; 9] = [
    "Aston Martin",
    "Land Rover",
    "Range Rover",
    "Ineos Grenadier",
    "Alfa Romeo",
    "Rolls-Royce",
    "Rolls Royce",
    "Great Wall",
    "Lynk & Co",
];

lazy_static! {
    static ref INIT_LOGGER: Once = Once::new();
    pub static ref NOW: DateTime<Utc> = chrono::Utc::now();
    pub static ref TIMESTAMP: i64 = NOW.timestamp();
    pub static ref CONFIG: AppConfig = AppConfig::from_file("config/config.yml");
    pub static ref LOG_CONFIG: String = format!("{}/meta_log4rs.yml", CONFIG.get_log4rs_config());
    pub static ref CREATED_ON: String = NOW.format(DATE_FORMAT).to_string();
    pub static ref BASE_INFO_CSV_FILE_NAME: String =
        format!("data/vehicles-info-{}.csv", CREATED_ON.clone());
    pub static ref DETAILS_CSV_NAME: String =
        format!("data/details-info-{}.csv", CREATED_ON.clone());
    pub static ref PRICES_CSV_FILE_NAME: String =
        format!("data/prices-info-{}.csv", CREATED_ON.clone());
    pub static ref BASE_INFO_PROTOBUF_FILE_NAME: String =
        format!("data/vehicles-info-{}.bin", CREATED_ON.clone());
    pub static ref DETAILS_PROTOBUF_NAME: String =
        format!("data/details-info-{}.bin", CREATED_ON.clone());
    pub static ref PRICES_PROTOBUF_FILE_NAME: String =
        format!("data/prices-info-{}.binpb", CREATED_ON.clone());
}
