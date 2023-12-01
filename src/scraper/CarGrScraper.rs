use std::{collections::HashMap, time::Duration};

use async_trait::async_trait;
use log::info;
use scraper::{Html, Selector};

use crate::{
    helpers::CarGrHTMLHelper::{get_listed_links, get_total_number},
    BROWSER_USER_AGENT,
};

use super::ScraperTrait::{LinkId, Scraper, ScraperTrait};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref REQWEST_ASYNC_CLIENT: reqwest::Client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .redirect(reqwest::redirect::Policy::limited(10))
        .user_agent(BROWSER_USER_AGENT)
        .build()
        .unwrap();
}

pub struct CarGrScraper {
    pub parent: Scraper,
}

impl CarGrScraper {
    pub fn new(url: &str, page: String, wait_time_ms: u64) -> Self {
        CarGrScraper {
            parent: Scraper::new(url, page, wait_time_ms),
        }
    }
}

#[async_trait]
impl ScraperTrait for CarGrScraper {
    async fn total_number(&self, params: HashMap<String, String>) -> Result<u32, String> {
        let url = self
            .parent
            .search_url(Some("/classifieds/cars/?".to_string()), params, 0)
            .to_owned();
        info!("url: {}", url);
        let response = REQWEST_ASYNC_CLIENT.get(url).send().await;

        match response {
            Ok(response) => match response.text().await {
                Ok(text) => {
                    let count = get_total_number(&text);
                    Ok(count)
                }
                Err(e) => Err(e.to_string()),
            },
            Err(e) => Err(e.to_string()),
        }
    }

    async fn get_listed_ids(
        &self,
        params: HashMap<String, String>,
        page_number: u32,
    ) -> Result<Vec<LinkId>, String> {
        let url = self.parent.search_url(
            Some("/classifieds/cars/?".to_string()),
            params.clone(),
            page_number,
        );
        let source = self.parent.html_search(url.as_str(), None).await?;
        let links = get_listed_links(&source);
        let mut ids = Vec::new();
        for link in links {
            let mut id = match link.split("/").last() {
                Some(id) => id.to_owned(),
                None => continue,
            };
            if id.contains("?") {
                id = id.split("?").collect::<Vec<_>>()[0].to_owned();
            }
            ids.push(LinkId {
                id,
                url: format!("{}{}", &self.parent.url, link),
            });
        }

        Ok(ids)
    }

    async fn parse_details(&self, link: LinkId) -> Result<HashMap<String, String>, String> {
        unimplemented!("Not implemented yet")
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
        helpers::CarGrHTMLHelper::get_total_number,
        scraper::{
            CarGrScraper::CarGrScraper,
            ScraperTrait::{Scraper, ScraperTrait},
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
        let scraper = CarGrScraper::new(url, "pg".to_owned(), 250);
        let total_number = scraper.total_number(params).await.unwrap();
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
        let scraper = CarGrScraper::new(url, "pg".to_owned(), 250);
        let ids = scraper.get_listed_ids(params, 1).await.unwrap();
        assert_eq!(24, ids.len());
        info!("ids: {:?}", ids);
    }
}
