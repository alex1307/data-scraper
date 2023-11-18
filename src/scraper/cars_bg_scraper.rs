use std::{collections::HashMap, time::Duration};

use log::info;
use reqwest::header::USER_AGENT;
use scraper::{Html, Selector};

use crate::scraper::mobile_bg::get_pages_async;

#[derive(Debug, Clone)]
struct CarsBG{
    url: String,
    client: reqwest::Client,
    total_number_selector: Selector,
    id_selector: Selector,
    wait_time_ms: u64,
}

impl CarsBG {
    pub fn new(url: &str, wait_time_ms: u64) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .unwrap();
        CarsBG {
            url: url.to_string(),
            client,
            wait_time_ms,
            total_number_selector: Selector::parse("span.milestoneNumberTotal").unwrap(),
            id_selector: Selector::parse("div.offer-item").unwrap(),
        }
    }

    fn search_url(&self, path: Option<&str>, params: &HashMap<String, String>, page: u32) -> String {

        let mut url = if let Some(path) = path  { 
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
    
        url = format!("{}page={}", url, page);
        url
    }

    pub async fn total_number(&self, params: HashMap<String, String>) -> Result<u32, String> {
        let url = self.search_url(Some("/carslist.php?"), &params, 0);
        let html = self.html_search(url.as_str()).await?;
        let document = Html::parse_document(&html);
        let element = document.select(&self.total_number_selector).next().unwrap();
        let total_number = element
            .inner_html()
            .chars()
            .filter(|&c| c.is_numeric())
            .collect::<String>()
            .parse::<u32>()
            .unwrap_or(0);
        info!("totalNumber: {}", total_number);
        Ok(total_number)
    }

    async fn html_search(&self, url: &str) -> Result<String, String> {
        let response = self
            .client
            .get(url)
            .send()
            .await;
        if let Ok(response) = response {
            if let Ok(html) = response.text().await {
                return Ok(html);
            }
        }
        Err(format!("Failed to get html from {}", url))
    }   

    pub async fn get_listed_ids(&self, params: &HashMap<String, String>,) -> Result<Vec<String>, String> {
        let total_number = self.total_number(params.clone()).await?;
        let number_of_pages: u32 = ((total_number / 20) + 1).try_into().map_err(|_| "Failed to convert total number of pages to u32")?;
        let mut ids = vec![];
        for page in 1..number_of_pages {
            tokio::time::sleep(Duration::from_millis(self.wait_time_ms)).await;
            let url = self.search_url(Some("/carslist.php?"), params, page);
            let html = self.html_search(url.as_str()).await?;
            let document = Html::parse_document(&html);
            for element in document.select(&self.id_selector) {
                if let Some(id) = element.value().attr("data-id") {
                    ids.push(id.to_owned());
                }
            }
        }
        
        Ok(ids)
    }

}