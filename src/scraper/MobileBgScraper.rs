use std::{sync::Arc, time::Duration};

use crate::{
    BROWSER_USER_AGENT,
    helpers::MobileBgHTMLHelper::process_html,
    model::{Search::Search, VehicleDataModel::Vehicle},
};
use async_trait::async_trait;
use lazy_static::lazy_static;
use log::error;
use regex::Regex;
use scraper::{Html, Selector};

use super::{
    BrowserController::BrowserController, Traits::Scraper, VehicleTraits::VehicleScrapeTrait,
};

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

#[async_trait]
impl VehicleScrapeTrait for MobileBGScraper {
    async fn get_html(&self, search: Search, page: u32) -> Result<String, String> {
        let url = self.get_search_url(search, page);
        self.parent
            .html_search(&url, Some("windows-1251".to_string()))
            .await
    }

    async fn browse_html(
        &self,
        browser: Arc<BrowserController>,
        search: Search,
        page: u32,
    ) -> Result<String, String> {
        let url = self.get_search_url(search, page);
        browser.get_html(&url)
    }

    async fn process_listed_results(
        &self,
        search: Search,
        page: u32,
    ) -> Result<Vec<Vehicle>, String> {
        let html = self.get_html(search.clone(), page).await?;
        let vehicles = self.process_html(&html)?;
        Ok(vehicles)
    }

    fn process_html(&self, html: &str) -> Result<Vec<Vehicle>, String> {
        let source = self.parent.source.clone();
        let vehicles: Vec<Vehicle> = process_html(html)
            .into_iter()
            .map(|mut v| {
                v.source = source.clone();
                Vehicle::from(v)
            })
            .collect();
        if vehicles.is_empty() {
            return Err("No vehicles found".to_string());
        }
        Ok(vehicles)
    }

    fn total_number(&self, html: &str) -> Result<u32, String> {
        let document = Html::parse_document(html);
        let selector = Selector::parse(r#"meta[name="description"]"#).unwrap();

        if let Some(element) = document.select(&selector).next() {
            if let Some(content) = element.value().attr("content") {
                let re = Regex::new(r"»\s*(\d+)\s*«").unwrap();
                if let Some(caps) = re.captures(content) {
                    if let Some(matched) = caps.get(1) {
                        let total_number = matched
                            .as_str()
                            .parse::<u32>()
                            .map_err(|_| "Failed to parse number from string".to_string());
                        return total_number;
                    }
                } else {
                    error!("Number not found");
                }
            }
        } else {
            error!("Total number not found");
        }

        Err("Number not found".to_string())
    }

    fn get_number_of_pages(&self, total_number: u32) -> Result<u32, String> {
        self.parent.get_number_of_pages(total_number)
    }

    fn get_search_url(&self, search: Search, page: u32) -> String {
        if page == 1 {
            search.url.replace("{page}", "")
        } else {
            search.url.replace("{page}", &format!("/p-{}", page))
        }
    }
    fn get_timeout(&self) -> u64 {
        250
    }
}
