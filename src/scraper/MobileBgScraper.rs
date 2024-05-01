use std::{str::FromStr, time::Duration};

use async_trait::async_trait;

use log::{error, info};
use rand::Rng;
use regex::Regex;
use scraper::{Html, Selector};
use tokio::time::sleep;

use super::Traits::{ScrapeListTrait, Scraper, ScraperTrait};
use crate::{
    helpers::MobileBgHTMLHelper::get_vehicles,
    model::{
        enums::{Engine, Gearbox},
        Search::Search,
        VehicleDataModel::ScrapedListData,
        VehicleRecord::MobileRecord,
    },
    BROWSER_USER_AGENT,
};
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
pub struct MobileBGScraper {
    pub parent: Scraper,
}

impl MobileBGScraper {
    pub fn new(url: &str, wait_time_ms: u64) -> Self {
        MobileBGScraper {
            parent: Scraper::new(url, "f1".to_string(), wait_time_ms),
        }
    }
}

#[async_trait]
impl ScrapeListTrait<MobileRecord> for MobileBGScraper {
    async fn process_listed_results(
        &self,
        search: Search,
        page_number: u32,
    ) -> Result<ScrapedListData<MobileRecord>, String> {
        let url = (&search.url).to_string();
        let html = self
            .parent
            .html_search(&url, Some("windows-1251".to_string()))
            .await?;

        let value = search.gearbox.clone().unwrap().to_string();
        let gearbox = Gearbox::from_str(&value).unwrap();
        let value = search.engine.clone().unwrap().to_string();
        let engine = Engine::from_str(&value).unwrap();
        let power: u32 = search.power.clone().unwrap().parse().unwrap();
        let searchId = search.hash.clone();
        let source = search.source.clone();
        let mut vehicles = get_vehicles(&html);
        for vehicle in vehicles.iter_mut() {
            vehicle.gearbox = gearbox;
            vehicle.power = power;
            vehicle.engine = engine;
            vehicle.searchId = searchId.clone();
            vehicle.source = source.clone();
        }
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
        let waiting_time_ms: u64 = rand::thread_rng().gen_range(1_000..3_000);
        sleep(Duration::from_millis(waiting_time_ms as u64)).await;

        Ok(ScrapedListData::Values(vehicles))
    }
}

#[async_trait]
impl ScraperTrait for MobileBGScraper {
    async fn get_html(&self, search: Search, page: u32) -> Result<String, String> {
        let url = self.get_search_url(search, page);
        self.parent
            .html_search(&url, Some("windows-1251".to_string()))
            .await
    }

    fn total_number(&self, html: &str) -> Result<u32, String> {
        let document = Html::parse_document(html);
        let selector = Selector::parse(r#"meta[name="description"]"#).unwrap();

        if let Some(element) = document.select(&selector).next() {
            if let Some(content) = element.value().attr("content") {
                let re = Regex::new(r"(\d+) « предложения").unwrap();
                if let Some(caps) = re.captures(content) {
                    if let Some(matched) = caps.get(1) {
                        return matched
                            .as_str()
                            .parse::<u32>()
                            .map_err(|_| "Failed to parse number from string".to_string());
                    }
                }
            }
        }

        Err("Number not found".to_string())
    }

    fn get_number_of_pages(&self, total_number: u32) -> Result<u32, String> {
        self.parent.get_number_of_pages(total_number)
    }

    fn get_search_url(&self, search: Search, page: u32) -> String {
        if page == 1 {
            return search.url;
        }
        format!("{}/&p-{}", search.url, page)
    }
}

#[cfg(test)]
mod screaper_mobile_bg_test {
    use std::collections::HashMap;

    use crate::{
        model::{Search::Search, VehicleDataModel::ScrapedListData},
        scraper::{
            MobileBgScraper,
            Traits::{ScrapeListTrait, ScraperTrait as _},
        },
        utils::helpers::configure_log4rs,
        LOG_CONFIG,
    };
    use log::info;

    #[tokio::test]
    async fn total_number_test() {
        configure_log4rs(&LOG_CONFIG);
        let mobile_bg =
            MobileBgScraper::MobileBGScraper::new("https://www.mobile.bg/pcgi/mobile.cgi?", 250);
        let mut params = HashMap::new();
        params.insert("act".to_owned(), "3".to_owned());
        params.insert("f10".to_owned(), "2004".to_owned());
        params.insert("topmenu".to_string(), "1".to_string());
        params.insert("rub".to_string(), 1.to_string());
        params.insert("pubtype".to_string(), 1.to_string());
        params.insert("f7".to_string(), 10000.to_string());
        params.insert(
            "f94".to_string(),
            "1~%CA%E0%EF%E0%F0%E8%F0%E0%ED%5C%CF%F0%EE%E4%E0%E4%E5%ED".to_string(),
        );
        let search = Search::from(params.clone());
        let html = mobile_bg.get_html(search, 1).await.unwrap();
        let total_number = mobile_bg.total_number(&html).unwrap();
        params.clear();
        params.insert("act".to_owned(), "3".to_owned());
        params.insert("rub".to_string(), 1.to_string());
        params.insert("pubtype".to_string(), 1.to_string());
        params.insert("topmenu".to_string(), "1".to_string());
        let search = Search::from(params.clone());
        let html = mobile_bg.get_html(search.clone(), 1).await.unwrap();
        let slink_totals = mobile_bg.total_number(&html).unwrap();

        assert_eq!(total_number, slink_totals);

        let number_of_pages = mobile_bg.parent.get_number_of_pages(total_number).unwrap();
        let mut all = vec![];
        for page in 1..number_of_pages + 1 {
            let data = mobile_bg
                .process_listed_results(search.clone(), page)
                .await
                .unwrap();
            match data {
                ScrapedListData::Values(ids) => {
                    assert!(ids.len() > 0);
                    all.extend(ids);
                }
                ScrapedListData::Error(error) => {
                    info!("error: {}", error);
                }
                ScrapedListData::SingleValue(link) => {
                    info!("link: {:?}", link);
                }
            }
        }
        assert_eq!(all.len(), total_number as usize);
    }
}
