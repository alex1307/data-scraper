use std::{sync::Arc, time::Duration};

use crate::{
    BROWSER_USER_AGENT,
    helpers::MobileBgHTMLHelper::process_html,
    model::{Search::Search, VehicleDataModel::Vehicle},
};
use async_trait::async_trait;
use lazy_static::lazy_static;
use log::{error, info};
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
        info!("MobileBGScraper GET (reqwest): {}", url);
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
        info!("MobileBGScraper GET (browser): {}", url);
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
                // Match the number inside the guillemets: » 1 910 « (digits may contain spaces or NBSP)
                let re = Regex::new(r"»\s*([\d\s\u{00A0}]+)\s*«").unwrap();
                if let Some(caps) = re.captures(content) {
                    if let Some(matched) = caps.get(1) {
                        let raw = matched.as_str();
                        // Keep only ASCII digits; drop spaces and NBSP
                        let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
                        return digits
                            .parse::<u32>()
                            .map_err(|_| format!("Failed to parse number from '{}'", raw));
                    }
                } else {
                    error!("Number not found in meta description: {}", content);
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
        let mut url = search.url.clone();

        // Prefer explicit `p-{page}` pattern used by MobileBgFilterService
        if url.contains("p-{page}") {
            if page <= 1 {
                // remove the whole placeholder segment (both variants)
                url = url.replace("/p-{page}", "");
                url = url.replace("p-{page}", "");
            } else {
                url = url.replace("p-{page}", &format!("p-{}", page));
            }
            info!("MobileBGScraper composed URL (page {}): {}", page, url);
            return url;
        }

        // Generic fallback: raw `{page}` placeholder
        if url.contains("{page}") {
            if page <= 1 {
                url = url.replace("{page}", "");
            } else {
                url = url.replace("{page}", &page.to_string());
            }
            info!("MobileBGScraper composed URL (page {}): {}", page, url);
            return url;
        }

        // Already concrete URL (no placeholders)
        info!("MobileBGScraper composed URL (page {}): {}", page, url);
        url
    }
    fn get_timeout(&self) -> u64 {
        250
    }
}
