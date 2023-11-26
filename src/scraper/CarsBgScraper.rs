use std::{collections::HashMap, time::Duration};

use async_trait::async_trait;
use lazy_static::lazy_static;
use log::{debug, error, info};

use scraper::{Html, Selector};
use serde::Deserialize;

use crate::{
    scraper::{cars_bg_helpers::read_carsbg_details, ScraperTrait::LinkId},
    BROWSER_USER_AGENT,
};

use super::ScraperTrait::{Scraper, ScraperTrait};

lazy_static! {
    pub static ref REQWEST_ASYNC_CLIENT: reqwest::Client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .user_agent(BROWSER_USER_AGENT)
        .build()
        .unwrap();
}

#[derive(Debug, Clone, Deserialize)]
struct ViewCountsCarsBG {
    _status: String,
    value_resettable: u32,
}

pub async fn get_view_count(id: String) -> Result<u32, String> {
    // let url = format!("https://stats.cars.bg/add/?object_id={}", id);
    // // match REQWEST_ASYNC_CLIENT.get(url).send().await {
    // //     Ok(_) => (),
    // //     Err(e) => {
    // //         error!(
    // //             "Error setting counter for: {}. Error: {}",
    // //             id,
    // //             e.to_string()
    // //         );
    // //         return Ok(0);
    // //     }
    // // };
    let url = format!("https://stats.cars.bg/get/?object_id={}", id);
    let response = REQWEST_ASYNC_CLIENT.get(url).send().await;

    match response {
        Ok(response) => match response.json::<ViewCountsCarsBG>().await {
            Ok(views) => Ok(views.value_resettable),
            Err(e) => {
                error!(
                    "Error setting counter for: {}. Error: {}",
                    id,
                    e.to_string()
                );
                Ok(0)
            }
        },
        Err(e) => Err(e.to_string()),
    }
}

#[derive(Debug, Clone)]
pub struct CarsBGScraper {
    pub parent: Scraper,
}

impl CarsBGScraper {
    pub fn new(url: &str, wait_time_ms: u64) -> Self {
        CarsBGScraper {
            parent: Scraper::new(url, "page".to_string(), wait_time_ms),
        }
    }
}

#[async_trait]
impl ScraperTrait for CarsBGScraper {
    async fn total_number(&self, params: HashMap<String, String>) -> Result<u32, String> {
        let url = self
            .parent
            .search_url(Some("/carslist.php?".to_string()), params.clone(), 0);
        let html = self.parent.html_search(url.as_str(), None).await?;
        let document = Html::parse_document(&html);
        let total_number_selector = Selector::parse("span.milestoneNumberTotal").unwrap();
        let element = document.select(&total_number_selector).next().unwrap();
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

    async fn get_listed_ids(
        &self,
        params: HashMap<String, String>,
        page_number: u32,
    ) -> Result<Vec<LinkId>, String> {
        let mut ids = vec![];
        tokio::time::sleep(Duration::from_millis(self.parent.wait_time_ms)).await;
        let url = self.parent.search_url(
            Some("/carslist.php?".to_string()),
            params.clone(),
            page_number,
        );
        let html = self.parent.html_search(url.as_str(), None).await?;
        let document = Html::parse_document(&html);
        let selector = Selector::parse("div.mdc-card__primary-action").unwrap();
        for element in document.select(&selector) {
            let html_fragment = Html::parse_fragment(element.inner_html().as_str());
            let selector = Selector::parse("a").unwrap();
            for e in html_fragment.select(&selector) {
                if let Some(href) = e.value().attr("href") {
                    if let Some(id) = href.split("/offer/").last() {
                        ids.push(LinkId {
                            url: href.to_string(),
                            id: id.to_owned(),
                        });
                        break;
                    }
                }
            }
        }
        debug!("ids: {:?}", ids);
        Ok(ids)
    }

    async fn parse_details(&self, link: LinkId) -> Result<HashMap<String, String>, String> {
        let html = self.parent.html_search(&link.url, None).await?;
        let mut result = read_carsbg_details(html);
        match get_view_count(link.id.clone()).await {
            Ok(views) => {
                result.insert("view_count".to_owned(), views.to_string());
            }
            Err(e) => {
                error!(
                    "Error setting counter for: {}. Error: {}",
                    link.id,
                    e.to_string()
                );
            }
        }
        result.insert("id".to_owned(), link.id);
        Ok(result)
    }

    fn get_number_of_pages(&self, total_number: u32) -> Result<u32, String> {
        self.parent.get_number_of_pages(total_number)
    }
}

#[cfg(test)]
mod cars_bg_tests {
    use std::collections::HashMap;

    use log::info;

    use crate::{
        scraper::{CarsBgScraper::CarsBGScraper, ScraperTrait::ScraperTrait as _},
        utils::helpers::configure_log4rs,
        LOG_CONFIG,
    };

    #[tokio::test]
    async fn total_number_test() {
        configure_log4rs(&LOG_CONFIG);
        let cars_bg = CarsBGScraper::new("https://www.cars.bg", 250);
        let mut params = HashMap::new();
        //subm=1&add_search=1&typeoffer=1&priceFrom=18000&priceTo=30000&yearFrom=2007&yearTo=2011&page=32
        params.insert("subm".to_owned(), "1".to_owned());
        params.insert("add_search".to_owned(), "1".to_owned());
        params.insert("typeoffer".to_owned(), "1".to_owned());
        params.insert("priceFrom".to_owned(), "25000".to_owned());
        params.insert("priceTo".to_owned(), "30000".to_owned());
        params.insert("yearFrom".to_owned(), "2010".to_owned());
        params.insert("yearTo".to_owned(), "2011".to_owned());

        let total_number = cars_bg.total_number(params.clone()).await.unwrap();
        assert!(total_number > 0);
        info!("total_number: {}", total_number);
        let number_of_pages = cars_bg.parent.get_number_of_pages(total_number).unwrap();
        assert_eq!(number_of_pages, total_number / 20 + 1);
        let mut all = vec![];
        for page in 1..=number_of_pages + 1 {
            let ids = cars_bg.get_listed_ids(params.clone(), page).await.unwrap();
            all.extend(ids);
        }

        assert!(all.len() > 0);
        assert_eq!(all.len(), total_number as usize);
        info!("all: {:?}", all);
    }

    #[tokio::test]
    async fn read_details_test() {
        configure_log4rs(&LOG_CONFIG);
        let cars_bg = CarsBGScraper::new("https://www.cars.bg", 250);
        let mut params = HashMap::new();
        //subm=1&add_search=1&typeoffer=1&priceFrom=18000&priceTo=30000&yearFrom=2007&yearTo=2011&page=32
        params.insert("subm".to_owned(), "1".to_owned());
        params.insert("add_search".to_owned(), "1".to_owned());
        params.insert("typeoffer".to_owned(), "1".to_owned());
        params.insert("priceFrom".to_owned(), "25000".to_owned());
        params.insert("priceTo".to_owned(), "30000".to_owned());
        params.insert("yearFrom".to_owned(), "2010".to_owned());
        params.insert("yearTo".to_owned(), "2011".to_owned());

        let ids = cars_bg.get_listed_ids(params.clone(), 1).await.unwrap();
        let first = ids.first().unwrap();
        let path = Some(format!("/offer/{:?}", first));
        let search_url = cars_bg.parent.search_url(path, HashMap::new(), 0);
        info!("search_url: {}", search_url);
        let details = cars_bg.parse_details(first.clone()).await.unwrap();
        assert!(details.len() > 0);
        assert_eq!(details.get("id").unwrap(), &first.id);
        info!("details: {:?}", details);
        let record = crate::model::records::MobileRecord::from(details);
        info!("record: {:?}", record);
    }
}
