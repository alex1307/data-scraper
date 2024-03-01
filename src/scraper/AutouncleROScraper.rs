use std::{collections::HashMap, time::Duration};

use crate::{
    helpers::AutoUncleHelper::get_vehicles,
    model::{AutoUncleVehicle::AutoUncleVehicle, VehicleDataModel::ScrapedListData},
    BROWSER_USER_AGENT,
};

use super::Traits::{ScrapeListTrait, Scraper, ScraperTrait};
use async_trait::async_trait;
use lazy_static::lazy_static;
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
pub struct AutouncleROScraper {
    pub parent: Scraper,
}

impl AutouncleROScraper {
    pub fn new(url: &str, wait_time_ms: u64) -> Self {
        AutouncleROScraper {
            parent: Scraper::new(url, "page".to_string(), wait_time_ms),
        }
    }
}

#[async_trait]
impl ScrapeListTrait<AutoUncleVehicle> for AutouncleROScraper {
    async fn process_listed_results(
        &self,
        params: HashMap<String, String>,
        page_number: u32,
    ) -> Result<ScrapedListData<AutoUncleVehicle>, String> {
        let html = self.get_html(params.clone(), page_number).await?;
        let mut vehicles = get_vehicles(&html);
        if vehicles.is_empty() {
            panic!("{}", html);
        }
        for v in &mut vehicles {
            v.source = "autouncle.ro".to_string();
        }
        let waiting_time_ms: u64 = rand::thread_rng().gen_range(5_000..8_000);
        sleep(Duration::from_millis(waiting_time_ms as u64)).await;
        Ok(ScrapedListData::Values(vehicles))
    }
}
#[async_trait]
impl ScraperTrait for AutouncleROScraper {
    async fn get_html(&self, params: HashMap<String, String>, page: u32) -> Result<String, String> {
        let url = self.parent.search_url(self.get_search_path(), params, page);
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

    use crate::{utils::helpers::configure_log4rs, LOG_CONFIG};

    use super::*;

    #[test]
    fn test_get_number_of_pages() {
        configure_log4rs(&LOG_CONFIG);
        let autouncle = AutouncleROScraper::new("https://www.autouncle.ro/en/cars_search", 0);
        let start = Instant::now();
        let content = fs::read_to_string("resources/test-data/autouncle/2.html").unwrap();
        let number = autouncle.total_number(&content).unwrap();
        assert_eq!(number, 84909);
        info!("Time: {:?}", start.elapsed());
    }
}
