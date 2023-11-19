use std::collections::HashMap;

use async_trait::async_trait;
use encoding_rs::{Encoding, UTF_8};
use lazy_static::lazy_static;

use crate::BROWSER_USER_AGENT;

lazy_static! {
    pub static ref REQWEST_ASYNC_CLIENT: reqwest::Client = reqwest::Client::builder()
        .user_agent(BROWSER_USER_AGENT)
        .build()
        .unwrap();
}

#[async_trait]
pub trait ScraperTrait {
    async fn total_number(&self, params: HashMap<String, String>) -> Result<u32, String>;
    async fn get_listed_ids(
        &self,
        params: HashMap<String, String>,
        page_number: u32,
    ) -> Result<Vec<String>, String>;
    async fn parse_details(
        &self,
        url: String,
        id: String,
    ) -> Result<HashMap<String, String>, String>;
}

#[derive(Debug, Clone)]
pub struct Scraper {
    url: String,
    page: String,
    pub wait_time_ms: u64,
}
impl Scraper {
    pub fn new(url: &str, page: String, wait_time_ms: u64) -> Self {
        Scraper {
            url: url.to_string(),
            page,
            wait_time_ms,
        }
    }

    pub fn search_url(
        &self,
        path: Option<String>,
        params: HashMap<String, String>,
        page: u32,
    ) -> String {
        let mut url = if let Some(path) = path {
            format!("{}{}", self.url, path)
        } else {
            self.url.clone()
        };

        if params.is_empty() {
            return url;
        }

        for (key, value) in params.iter() {
            url = format!("{}{}={}&", url, key, value);
        }
        if page == 0 {
            return url.trim_end_matches('&').to_owned();
        }

        url = format!("{}{}={}", url, self.page, page);
        url
    }

    pub fn get_number_of_pages(&self, total_number: u32) -> Result<u32, String> {
        if total_number == 0 {
            return Ok(0);
        }
        let number_of_pages: u32 = (total_number / 20)
            .try_into()
            .map_err(|_| "Failed to convert total number of pages to u32")?;
        if total_number % 20 == 0 {
            return Ok(number_of_pages);
        } else {
            return Ok(number_of_pages + 1);
        }
    }

    pub async fn html_search(
        &self,
        url: &str,
        decoding_from: Option<String>,
    ) -> Result<String, String> {
        let response = REQWEST_ASYNC_CLIENT.get(url).send().await;
        if let Ok(response) = response {
            if let Some(label) = decoding_from {
                let bytes = response.bytes().await.unwrap().to_vec();
                //"windows-1251"
                let encoding = Encoding::for_label(label.as_bytes()).unwrap();
                let (decoded, _, _) = encoding.decode(&bytes);
                let utf8_html = UTF_8.encode(&decoded).0;
                let response = String::from_utf8_lossy(&utf8_html);
                return Ok(response.to_string());
            } else if let Ok(html) = response.text().await {
                return Ok(html);
            }
        }
        Err(format!("Failed to get html from {}", url))
    }
}
