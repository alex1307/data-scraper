use crate::{
    helpers::AutoUncleHelper::process_html as process_autouncle_html,
    model::{Search::Search, VehicleDataModel::Vehicle},
};
use async_trait::async_trait;

use rand::Rng;

use std::{fmt::Debug, time::Duration};
use tokio::time::sleep;

use super::{Traits::Scraper, VehicleTraits::VehicleScrapeTrait};
#[derive(Debug, Clone)]
pub struct AutouncleScraper {
    pub parent: super::Traits::Scraper,
}

impl AutouncleScraper {
    pub fn new(url: &str, page: &str, source: &str, wait_time_ms: u64) -> Self {
        AutouncleScraper {
            parent: Scraper::new(url, page.to_string(), source.to_string(), wait_time_ms),
        }
    }
}

#[async_trait]
impl VehicleScrapeTrait for AutouncleScraper {
    async fn process_listed_results(
        &self,
        search: Search,
        page: u32,
    ) -> Result<Vec<Vehicle>, String> {
        let html = self.get_html(search.clone(), page).await?;
        let vehicles = self.process_html(&html)?;

        let waiting_time_ms: u64 = rand::rng().random_range(3_000..5_000);
        log::info!("Sleeping for{} millis", waiting_time_ms);
        sleep(Duration::from_millis(waiting_time_ms as u64)).await;
        log::info!("keeping on...");
        Ok(vehicles)
    }

    fn process_html(&self, html: &str) -> Result<Vec<Vehicle>, String> {
        let source = self.parent.source.clone();
        let vehicles: Vec<Vehicle> = process_autouncle_html(html)
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
    async fn get_html(&self, search: Search, page: u32) -> Result<String, String> {
        let url = self.get_search_url(search, page);
        log::info!("URL: {}", url);
        self.parent.html_search(&url, None).await
    }

    async fn browse_html(
        &self,
        browser: std::sync::Arc<super::BrowserController::BrowserController>,
        search: Search,
        page: u32,
    ) -> Result<String, String> {
        let url = self.get_search_url(search, page);
        log::info!("URL: {}", url);
        browser.get_html(&url)
    }

    fn total_number(&self, html: &str) -> Result<u32, String> {
        let number_of_cars = match html.find(r#"\"numberOfCars\":"#) {
            Some(index) => {
                let index = index + r#"\"numberOfCars\":"#.len();
                let end_index = html[index..].find(',').unwrap();
                html[index..index + end_index].parse::<u32>().unwrap()
            }
            None => return Err("Not found".to_string()),
        };
        Ok(number_of_cars)
    }

    fn get_number_of_pages(&self, total_number: u32) -> Result<u32, String> {
        let number_of_pages = (total_number as f32 / 25.0).ceil() as u32;
        Ok(number_of_pages)
    }
}
