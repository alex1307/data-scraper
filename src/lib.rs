pub mod config;
pub mod downloader;
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
lazy_static! {
    static ref INIT_LOGGER: Once = Once::new();
}
