use std::collections::HashMap;

use async_trait::async_trait;
use encoding_rs::{Encoding, UTF_8};
use lazy_static::lazy_static;

use log::info;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

use crate::BROWSER_USER_AGENT;

lazy_static! {
    pub static ref REQWEST_ASYNC_CLIENT: reqwest::Client = reqwest::Client::builder()
        .user_agent(BROWSER_USER_AGENT)
        .build()
        .unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Default)]
pub struct LinkId {
    pub url: String,
    pub id: String,
}

impl PartialEq for LinkId {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for LinkId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[async_trait]
pub trait ScraperTrait {
    async fn get_listed_ids(
        &self,
        params: HashMap<String, String>,
        page_number: u32,
    ) -> Result<Vec<LinkId>, String>;
    async fn parse_details(&self, link: LinkId) -> Result<HashMap<String, String>, String>;

    fn total_number(&self, page: &str) -> Result<u32, String>;

    fn get_number_of_pages(&self, total_number: u32) -> Result<u32, String>;

    fn get_timeout(&self) -> u64 {
        250
    }

    async fn get_html(
        &self,
        path: Option<String>,
        params: HashMap<String, String>,
        page: u32,
    ) -> Result<String, String>;
}

#[derive(Debug, Clone)]
pub struct Scraper {
    pub url: String,
    page: String,
    pub headers: Vec<(String, String)>,
    pub wait_time_ms: u64,
}
impl Scraper {
    pub fn new(url: &str, page: String, wait_time_ms: u64) -> Self {
        Scraper {
            url: url.to_string(),
            page,
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
        self.headers = headers.clone();
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
            } else if let Ok(html) = response.text().await {
                return Ok(html);
            }
        }
        return Err(format!("Failed to get html from {}", url));
    }
}
