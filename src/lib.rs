use log::info;

pub mod config;
pub mod mobile_scraper;
pub mod writer;

use lazy_static::lazy_static;
use std::sync::Once;

pub const LISTING_URL: &str = "//www.mobile.bg/pcgi/mobile.cgi?act=3";
pub const DETAILS_URL: &str = "//www.mobile.bg/pcgi/mobile.cgi?act=4&adv={}&slink={}";

lazy_static! {
    static ref INIT_LOGGER: Once = Once::new();
}

pub fn configure_log4rs() -> () {
    INIT_LOGGER.call_once(|| {
        log4rs::init_file("config/log4rs.yml", Default::default()).unwrap();
        info!("SUCCESS: Loggers are configured with dir: _log/*");
    });
}

pub fn listing_url(slink: &str, page_number: i32) -> String {
    format!(
        "{}{}",
        LISTING_URL,
        format!("&slink={}&f1={}", slink, page_number)
    )
}

pub fn details_url(slink: &str, adv: &str) -> String {
    format!("{}{}", DETAILS_URL, format!("&slink={}&adv={}", slink, adv))
}
