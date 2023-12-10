use std::{collections::HashMap, thread::sleep, time::Duration};

use async_trait::async_trait;

use log::info;

use crate::{
    helpers::CarGrHTMLHelper::{get_listed_links, get_total_number, vehicle_data},
    model::VehicleDataModel::{LinkId, ScrapedListData},
    BROWSER_USER_AGENT,
};

use super::Traits::{RequestResponseTrait, ScrapeListTrait, Scraper, ScraperTrait};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref REQWEST_ASYNC_CLIENT: reqwest::Client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .redirect(reqwest::redirect::Policy::limited(10))
        .user_agent(BROWSER_USER_AGENT)
        .build()
        .unwrap();
}

#[derive(Debug, Clone)]
pub struct CarGrScraper {
    pub parent: Scraper,
}

impl CarGrScraper {
    pub fn new(url: &str, wait_time_ms: u64) -> Self {
        CarGrScraper {
            parent: Scraper::new(url, "pg".to_string(), wait_time_ms),
        }
    }
}

#[async_trait]
impl ScrapeListTrait<LinkId> for CarGrScraper {
    async fn get_listed_ids(
        &self,
        params: HashMap<String, String>,
    ) -> Result<ScrapedListData<LinkId>, String> {
        let html = self.get_html(params.clone(), 1).await?;
        let total_number = self.total_number(&html)?;
        let number_of_pages = self.get_number_of_pages(total_number)?;
        let mut list = vec![];
        for page_number in 1..number_of_pages + 1 {
            let url = self.parent.search_url(
                Some("/classifieds/cars/?".to_string()),
                params.clone(),
                page_number,
            );
            let source = self.parent.html_search(url.as_str(), None).await?;
            let links = get_listed_links(&source);
            for link in links {
                let mut id = match link.split("/").last() {
                    Some(id) => id.to_owned(),
                    None => continue,
                };
                if id.contains("?") {
                    id = id.split("?").collect::<Vec<_>>()[0].to_owned();
                }
                list.push(LinkId {
                    id,
                    url: format!("{}{}&lang=en?lang=en", &self.parent.url, link),
                });
            }
        }
        Ok(ScrapedListData::Values(list))
    }
}

#[async_trait]
impl RequestResponseTrait<LinkId, HashMap<String, String>> for CarGrScraper {
    async fn handle_request(&self, link: LinkId) -> Result<HashMap<String, String>, String> {
        let html = self.parent.html_search(&link.url, None).await?;
        info!("link: {:?}. String (len): {}", link, html.len());
        if html.len() < 2000 {
            info!("-------------------");
            info!("{}", html);
            sleep(Duration::from_secs(5));
            return Ok(HashMap::new());
        }
        let mut result = vehicle_data(&html);
        result.insert("id".to_owned(), link.id);
        info!("VEHICLE: {:?}", result);
        Ok(result)
    }
}

#[async_trait]
impl ScraperTrait for CarGrScraper {
    async fn get_html(&self, params: HashMap<String, String>, page: u32) -> Result<String, String> {
        let url = self.parent.search_url(self.get_search_path(), params, page);
        self.parent.html_search(&url, None).await
    }

    fn total_number(&self, html: &str) -> Result<u32, String> {
        let count = get_total_number(&html);
        Ok(count)
    }

    fn get_number_of_pages(&self, total_number: u32) -> Result<u32, String> {
        let number_of_pages = (total_number as f32 / 25.0).ceil() as u32;
        Ok(number_of_pages)
    }
}

#[cfg(test)]
mod car_gr_test_suit {
    use std::collections::HashMap;

    use log::info;

    use crate::{
        model::VehicleDataModel::ScrapedListData,
        scraper::{
            CarGrScraper::CarGrScraper,
            Traits::{ScrapeListTrait, ScraperTrait},
        },
        utils::helpers::configure_log4rs,
        LOG_CONFIG,
    };

    #[tokio::test]
    async fn get_listes_vehicles_test() {
        configure_log4rs(&LOG_CONFIG);
        let url = "https://www.car.gr";
        let mut params = HashMap::new();
        params.insert("lang".to_owned(), "en".to_owned());
        params.insert("category".to_owned(), "15001".to_owned());
        params.insert("price-from".to_owned(), "25000".to_owned());
        params.insert("price-to".to_owned(), "30000".to_owned());
        params.insert("registration-from".to_owned(), "2010".to_owned());
        params.insert("registration-to".to_owned(), "2011".to_owned());
        let scraper = CarGrScraper::new(url, 250);
        let url = scraper
            .parent
            .search_url(Some("/classifieds/cars/?".to_string()), params, 1);
        let html = scraper
            .parent
            .html_search(url.as_str(), None)
            .await
            .unwrap();
        let total_number = scraper.total_number(&html).unwrap();
        info!("total_number: {}", total_number);
    }

    #[tokio::test]
    async fn get_ids_test() {
        configure_log4rs(&LOG_CONFIG);
        let url = "https://www.car.gr";
        let mut params = HashMap::new();
        params.insert("lang".to_owned(), "en".to_owned());
        params.insert("category".to_owned(), "15001".to_owned());
        params.insert("price-from".to_owned(), "25000".to_owned());
        params.insert("price-to".to_owned(), "30000".to_owned());
        params.insert("registration-from".to_owned(), "2010".to_owned());
        params.insert("registration-to".to_owned(), "2011".to_owned());
        let scraper = CarGrScraper::new(url, 250);
        let ids = scraper.get_listed_ids(params).await.unwrap();
        if let ScrapedListData::Values(ids) = ids {
            info!("ids: {:?}", ids);
            assert_eq!(25, ids.len());
        }
    }
}
