use std::time::Duration;

use crate::BROWSER_USER_AGENT;
use lazy_static::lazy_static;

use super::Traits::Scraper;

lazy_static! {
    pub static ref REQWEST_ASYNC_CLIENT: reqwest::Client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .redirect(reqwest::redirect::Policy::limited(10))
        .user_agent(BROWSER_USER_AGENT)
        .build()
        .unwrap();
}

#[derive(Debug, Clone)]
pub struct MobileBGScraper {
    pub parent: Scraper,
}

impl MobileBGScraper {
    pub fn new(url: &str, wait_time_ms: u64) -> Self {
        MobileBGScraper {
            parent: Scraper::new(url, "f1".to_string(), "mobile.bg".to_string(), wait_time_ms),
        }
    }
}
