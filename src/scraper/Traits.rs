use std::{collections::HashMap, fmt::Debug};

use encoding_rs::{Encoding, UTF_8};
use lazy_static::lazy_static;

use log::info;
use rand::Rng;

use crate::{BROWSER_USER_AGENT, services::SearchBuilder::EXCLUED};

lazy_static! {
    pub static ref REQWEST_ASYNC_CLIENT: reqwest::Client = reqwest::Client::builder()
        .user_agent(BROWSER_USER_AGENT)
        .build()
        .unwrap();
}

#[derive(Debug, Clone)]
pub struct Scraper {
    pub url: String,
    pub page: String,
    pub source: String,
    pub headers: Vec<(String, String)>,
    pub wait_time_ms: u64,
}
impl Scraper {
    pub fn new(url: &str, page: String, source: String, wait_time_ms: u64) -> Self {
        Scraper {
            url: url.to_string(),
            page,
            source,
            wait_time_ms,
            headers: vec![],
        }
    }

    pub async fn headers(&mut self, url: &str, filter: HashMap<String, String>) {
        let response = REQWEST_ASYNC_CLIENT.get(url).send().await.unwrap();
        let mut headers = vec![];
        for header in response.headers().keys() {
            if filter.contains_key(&header.to_string()) {
                let new_header_key = filter.get(&header.to_string()).unwrap().to_owned();
                match response.headers().get(header) {
                    Some(value) => {
                        headers.push((new_header_key, value.to_str().unwrap().to_owned()))
                    }
                    None => {
                        info!("Header not found: {}", header.as_str());
                    }
                }
            }
        }
        self.headers.clone_from(&headers);
    }

    pub fn search_url(&self, params: HashMap<String, String>, page: u32) -> String {
        let mut url = self.url.clone();

        if params.is_empty() {
            return url;
        }

        for (key, value) in params.iter() {
            if EXCLUED.contains(&key.as_str()) {
                continue;
            }
            if value.contains('[') && value.contains(']') {
                let value = value.replace(['[', ']'], "");
                let values: Vec<&str> = value.split(',').collect();
                for value in values {
                    url = format!("{}{}={}&", url, key, value);
                }
                continue;
            }
            url = format!("{}{}={}&", url, key, value);
        }

        if page == 0 {
            return url.trim_end_matches('&').to_owned();
        }

        url = format!("{}{}={}", url, self.page, page);
        info!("URL: {}", url);
        url
    }

    pub fn get_number_of_pages(&self, total_number: u32) -> Result<u32, String> {
        if total_number == 0 {
            return Ok(0);
        }
        let number_of_pages: u32 = total_number / 20;
        if total_number % 20 == 0 {
            Ok(number_of_pages)
        } else {
            Ok(number_of_pages + 1)
        }
    }

    pub async fn html_search(
        &self,
        url: &str,
        decoding_from: Option<String>,
    ) -> Result<String, String> {
        let mut builder = REQWEST_ASYNC_CLIENT.get(url);
        let headers = self.headers.clone();
        for (key, value) in headers {
            builder = builder.header(key.clone(), value.clone());
            info!("Building HTTP request -> key: {}, value: {}", key, value);
        }
        let response = builder.send().await;
        if let Ok(response) = response {
            if let Some(label) = decoding_from {
                let bytes = response.bytes().await.unwrap().to_vec();
                //"windows-1251"
                let encoding = Encoding::for_label(label.as_bytes()).unwrap();
                let (decoded, _, _) = encoding.decode(&bytes);
                let utf8_html = UTF_8.encode(&decoded).0;
                let response = String::from_utf8_lossy(&utf8_html);
                return Ok(response.to_string());
            }
            if let Ok(html) = response.text().await {
                return Ok(html);
            }
        }
        Err(format!("Failed to get html from {}", url))
    }

    pub fn waiting_time(&self) -> u64 {
        rand::rng().random_range(1_000..3_000)
    }
}
