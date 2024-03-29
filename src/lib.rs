pub mod config;
pub mod model;
pub mod scraper;
pub mod services;
pub mod utils;
pub mod writer;

use lazy_static::lazy_static;

use std::sync::Once;

use crate::config::app_config::AppConfig;

pub const LISTING_URL: &str = "https://www.mobile.bg/pcgi/mobile.cgi?act=3&";
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

pub const BROWSER_USER_AGENT: &str ="Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.1 Safari/605.1.15";

lazy_static! {
    static ref INIT_LOGGER: Once = Once::new();
    pub static ref TIMESTAMP: i64 = chrono::Local::now().timestamp();
    pub static ref CONFIG: AppConfig = AppConfig::from_file("config/config.yml");
    pub static ref LOG_CONFIG: String = format!("{}/meta_log4rs.yml", CONFIG.get_log4rs_config());
    pub static ref CREATED_ON: String = chrono::Utc::now().format(DATE_FORMAT).to_string();
    pub static ref ARCHIVE_FILE_NAME: String =
        format!("{}/vehicle.archive.csv", CONFIG.get_data_dir());
    pub static ref INSALE_FILE_NAME: String = format!(
        "{}/vehicle-{}.csv",
        CONFIG.get_data_dir(),
        CREATED_ON.clone()
    );
    pub static ref UPDATED_VEHICLES_FILE_NAME: String = format!(
        "{}/updated_vehicle-{}.csv",
        CONFIG.get_data_dir(),
        CREATED_ON.clone()
    );
    pub static ref METADATA_FILE_NAME: String = format!("{}/meta_data.csv", CONFIG.get_data_dir());
    pub static ref FOR_UPDATE_FILE_NAME: String =
        format!("{}/for_update.csv", CONFIG.get_data_dir());
    pub static ref UPDATED_FILE_NAME: String = format!("{}/updated.csv", CONFIG.get_data_dir());
    pub static ref DELETED_FILE_NAME: String = format!("{}/deleted.csv", CONFIG.get_data_dir());
}
