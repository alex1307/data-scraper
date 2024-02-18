use std::{collections::HashMap, str::FromStr, time::Duration};

use async_trait::async_trait;
use lazy_static::lazy_static;
use log::{error, info};

use scraper::{Html, Selector};
use serde::Deserialize;

use crate::{
    helpers::{
        CarsBgHTMLHelper::{read_carsbg_details, read_listing},
        ENGINE_KEY, GEARBOX_KEY, MAKE_KEY, MILEAGE_KEY, PRICE_KEY, YEAR_KEY,
    },
    model::{
        enums::Gearbox,
        records::MobileRecord,
        VehicleDataModel::{LinkId, ScrapedListData},
    },
    services::SearchBuilder::{CARS_BG_GEARBOX_ID, CARS_BG_POWER_FROM, CARS_BG_POWER_TO},
    BROWSER_USER_AGENT,
};

use super::Traits::{RequestResponseTrait, ScrapeListTrait, Scraper, ScraperTrait};

lazy_static! {
    pub static ref REQWEST_ASYNC_CLIENT: reqwest::Client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .user_agent(BROWSER_USER_AGENT)
        .build()
        .unwrap();
}

#[derive(Debug, Clone, Deserialize)]
struct ViewCountsCarsBG {
    value_resettable: u32,
}

pub async fn get_view_count(id: String) -> Result<u32, String> {
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
impl ScrapeListTrait<MobileRecord> for CarsBGScraper {
    async fn process_listed_results(
        &self,
        params: HashMap<String, String>,
        page_number: u32,
    ) -> Result<ScrapedListData<MobileRecord>, String> {
        let url = self.parent.search_url(
            Some("/carslist.php?".to_string()),
            params.clone(),
            page_number,
        );
        let html = self.parent.html_search(url.as_str(), None).await?;
        let value = params.get("gearbox").unwrap().to_string();
        let gearbox = Gearbox::from_str(&value).unwrap();
        let power: u32 = params.get("power").unwrap().parse().unwrap();
        let vehicles = read_listing(html.as_str(), gearbox, power);
        Ok(ScrapedListData::Values(vehicles))
    }
}

// #[async_trait]
// impl RequestResponseTrait<LinkId, MobileRecord> for CarsBGScraper {
//     async fn handle_request(&self, link: LinkId) -> Result<MobileRecord, String> {
//         let html = self.parent.html_search(&link.url, None).await?;
//         let mut result = read_carsbg_details(html);
//         match get_view_count(link.id.clone()).await {
//             Ok(views) => {
//                 result.insert("view_count".to_owned(), views.to_string());
//             }
//             Err(e) => {
//                 error!(
//                     "Error setting counter for: {}. Error: {}",
//                     link.id,
//                     e.to_string()
//                 );
//             }
//         }
//         result.insert("id".to_owned(), link.id.clone());
//         if result.get(PRICE_KEY.to_string().as_str()).is_none() {
//             Err(format!("invalid/incompete PRICE for: {}", &link.id))
//         } else if result.get(MAKE_KEY.to_string().as_str()).is_none() {
//             Err(format!("invalid/incompete MAKE/MODEL for: {}", &link.id))
//         } else if result.get(YEAR_KEY.to_string().as_str()).is_none() {
//             Err(format!("invalid/incompete YEAR for: {}", &link.id))
//         } else if result.get(MILEAGE_KEY.to_string().as_str()).is_none() {
//             Err(format!("invalid/incompete MILEAGE for: {}", &link.id))
//         } else if result.get(ENGINE_KEY.to_string().as_str()).is_none() {
//             Err(format!("invalid/incompete ENGINE for: {}", &link.id))
//         } else if result.get(GEARBOX_KEY.to_string().as_str()).is_none() {
//             Err(format!("invalid/incompete GEARBOX for: {}", &link.id))
//         } else {
//             let record = MobileRecord::from(result);
//             Ok(record)
//         }
//     }
// }

#[async_trait]
impl ScraperTrait for CarsBGScraper {
    async fn get_html(&self, params: HashMap<String, String>, page: u32) -> Result<String, String> {
        let url = self.parent.search_url(self.get_search_path(), params, page);
        self.parent.html_search(&url, None).await
    }

    fn total_number(
        &self,
        html: &str,
        //,
    ) -> Result<u32, String> {
        let document = Html::parse_document(html);
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

    fn get_number_of_pages(&self, total_number: u32) -> Result<u32, String> {
        self.parent.get_number_of_pages(total_number)
    }

    fn get_search_path(&self) -> Option<String> {
        Some("/carslist.php?".to_string())
    }
}

#[cfg(test)]
mod cars_bg_tests {
    use std::collections::HashMap;

    use log::info;

    use crate::{
        model::VehicleDataModel::ScrapedListData,
        scraper::{
            CarsBgScraper::CarsBGScraper,
            Traits::{RequestResponseTrait, ScrapeListTrait, ScraperTrait as _},
        },
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
        let url = cars_bg
            .parent
            .search_url(Some("/carslist.php?".to_string()), params.clone(), 1);
        let html = cars_bg
            .parent
            .html_search(url.as_str(), None)
            .await
            .unwrap();
        let total_number = cars_bg.total_number(&html).unwrap();
        assert!(total_number > 0);
        info!("total_number: {}", total_number);
        let data = cars_bg
            .process_listed_results(params.clone(), 1)
            .await
            .unwrap();
        match data {
            ScrapedListData::Values(ids) => {
                assert!(ids.len() > 0);
                info!("ids: {:?}", ids);
                assert_eq!(ids.len(), total_number as usize);
            }
            _ => panic!("Wrong data type"),
        }
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

        let data = cars_bg
            .process_listed_results(params.clone(), 1)
            .await
            .unwrap();
        match data {
            ScrapedListData::Values(ids) => {
                assert!(ids.len() > 0);
                info!("ids: {:?}", ids);
                let first = ids.first().unwrap();
                let path = Some(format!("/offer/{:?}", first));
                let search_url = cars_bg.parent.search_url(path, HashMap::new(), 0);
                info!("search_url: {}", search_url);
                // let record = cars_bg.handle_request(first.clone()).await.unwrap();
                // info!("record: {:?}", record);
            }
            _ => panic!("Wrong data type"),
        }
    }
}
