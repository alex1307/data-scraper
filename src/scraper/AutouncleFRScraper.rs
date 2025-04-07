use std::time::Duration;

use crate::{
    BROWSER_USER_AGENT,
    helpers::AutoUncleHelper::process_html,
    model::{AutouncleJsonModel::CarData, Search::Search, VehicleDataModel::ScrapedListData},
};

use super::Traits::{ScrapeListTrait, Scraper, ScraperTrait};
use async_trait::async_trait;
use lazy_static::lazy_static;
use log::{error, info};
use rand::Rng;
use tokio::time::sleep;

lazy_static! {
    pub static ref REQWEST_ASYNC_CLIENT: reqwest::Client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .redirect(reqwest::redirect::Policy::limited(10))
        .user_agent(BROWSER_USER_AGENT)
        .build()
        .unwrap();
}

#[derive(Debug, Clone)]
pub struct AutouncleFRScraper {
    pub parent: Scraper,
}

impl AutouncleFRScraper {
    pub fn new(url: &str, wait_time_ms: u64) -> Self {
        AutouncleFRScraper {
            parent: Scraper::new(
                url,
                "page".to_string(),
                "autouncle.fr".to_string(),
                wait_time_ms,
            ),
        }
    }
}

#[async_trait]
impl ScrapeListTrait<CarData> for AutouncleFRScraper {
    async fn process_listed_results(
        &self,
        search: Search,
        page_number: u32,
    ) -> Result<ScrapedListData<CarData>, String> {
        let html = self.get_html(search.clone(), page_number).await?;
        let mut vehicles = process_html(&html);
        if vehicles.is_empty() {
            if html.to_lowercase().contains("too many requests")
                || html.to_lowercase().contains(r#""429""#)
                || html.to_lowercase().contains(r#"429 "#)
                || html.to_lowercase().contains(r#" 429"#)
            {
                error!("429 - TOO MANY REQUESTS{}", html.len());
            } else {
                error!(
                    "No vehicles found. Page: {}, Search: {:?}",
                    page_number, search
                );
            }
            info!("*** Waiting 30 seconds ***");
            sleep(Duration::from_secs(30)).await;
        }
        for v in &mut vehicles {
            v.source = search.clone().source;
            v.searchId = search.clone().hash;
        }
        let waiting_time_ms: u64 = rand::rng().random_range(5_000..8_000);
        sleep(Duration::from_millis(waiting_time_ms as u64)).await;
        Ok(ScrapedListData::Values(vehicles))
    }
}
#[async_trait]
impl ScraperTrait for AutouncleFRScraper {
    async fn get_html(&self, search: Search, page: u32) -> Result<String, String> {
        let url = self.get_search_url(search, page);
        self.parent.html_search(&url, None).await
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

#[cfg(test)]
mod autouncle_test {

    use std::{fs, time::Instant};

    use log::info;

    use crate::{LOG_CONFIG, utils::helpers::configure_log4rs};

    use super::*;

    #[test]
    fn test_get_number_of_pages() {
        configure_log4rs(&LOG_CONFIG);
        let autouncle = AutouncleFRScraper::new("https://www.autouncle.nl/fr/cars_search", 0);
        let start = Instant::now();
        let content = fs::read_to_string("resources/test-data/autouncle/2.html").unwrap();
        let number = autouncle.total_number(&content).unwrap();
        assert_eq!(number, 84909);
        info!("Time: {:?}", start.elapsed());
    }
}
