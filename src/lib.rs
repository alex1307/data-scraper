use log::info;

pub mod mobile_scraper;
pub mod writer;

use lazy_static::lazy_static;
use std::sync::Once;

lazy_static! {
    static ref INIT_LOGGER: Once = Once::new();
}

pub fn configure_log4rs() -> () {
    INIT_LOGGER.call_once(|| {
        log4rs::init_file("config/log4rs.yml", Default::default()).unwrap();
        info!("SUCCESS: Loggers are configured with dir: _log/*");
    });
}
