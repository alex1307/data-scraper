use std::collections::HashMap;

use async_trait::async_trait;
use log::info;
use regex::Regex;
use scraper::{Html, Selector};

use crate::scraper::mobile_bg_helpers::slink;

use super::{
    mobile_bg_helpers::{details2map, get_url},
    scraper_trait::{Scraper, ScraperTrait},
};

pub struct MobileBGScraper {
    pub parent: Scraper,
    slink: Option<String>,
}

impl MobileBGScraper {
    pub fn new(url: &str, wait_time_ms: u64) -> Self {
        MobileBGScraper {
            parent: Scraper::new(url, "f1".to_string(), wait_time_ms),
            slink: None,
        }
    }

    pub async fn slink(&mut self, params: HashMap<String, String>) -> Result<String, String> {
        let url = self.parent.search_url(None, params.clone(), 0);
        let html = self
            .parent
            .html_search(url.as_str(), Some("windows-1251".to_string()))
            .await?;
        info!("html: {}", html);
        let slink = slink(&html);
        if slink.is_empty() {
            return Err("slink not found".to_string());
        }
        self.slink = Some(slink.clone());
        Ok(slink)
    }
}

#[async_trait]
impl ScraperTrait for MobileBGScraper {
    async fn total_number(&self, params: HashMap<String, String>) -> Result<u32, String> {
        let url = self.parent.search_url(None, params.clone(), 0);
        let html = self
            .parent
            .html_search(url.as_str(), Some("windows-1251".to_string()))
            .await?;
        let document = Html::parse_document(&html);
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

    async fn get_listed_ids(
        &self,
        params: HashMap<String, String>,
        page_number: u32,
    ) -> Result<Vec<String>, String> {
        let url = self.parent.search_url(None, params.clone(), page_number);
        let html = self
            .parent
            .html_search(&url, Some("windows-1251".to_string()))
            .await?;
        let document = Html::parse_document(&html);
        let mut links = vec![];
        let selector = Selector::parse("table.tablereset").unwrap();
        let re = Regex::new(r"adv=(\d+)").unwrap();
        for element in document.select(&selector) {
            if let Some(url) = get_url(&element) {
                if let Some(caps) = re.captures(&url) {
                    if let Some(matched) = caps.get(1) {
                        links.push(matched.as_str().to_string());
                    }
                }
            }
        }
        Ok(links)
    }

    async fn parse_details(
        &self,
        url: String,
        id: String,
    ) -> Result<HashMap<String, String>, String> {
        let html = self
            .parent
            .html_search(&url, Some("windows-1251".to_string()))
            .await?;
        let document = Html::parse_document(&html);
        let mut result = details2map(document);
        result.insert("id".to_owned(), id);
        Ok(result)
    }
}

#[cfg(test)]
mod screaper_mobile_bg_test {
    use std::collections::{HashMap, HashSet};

    use crate::{
        scraper::scraper_trait::ScraperTrait as _, utils::helpers::configure_log4rs, LOG_CONFIG,
    };
    use log::info;

    #[tokio::test]
    async fn total_number_test() {
        configure_log4rs(&LOG_CONFIG);
        let mut mobile_bg =
            super::MobileBGScraper::new("https://www.mobile.bg/pcgi/mobile.cgi?", 250);
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
        let total_number = mobile_bg.total_number(params.clone()).await.unwrap();
        let slink = mobile_bg.slink(params.clone()).await.unwrap();
        params.clear();
        params.insert("act".to_owned(), "3".to_owned());
        params.insert("rub".to_string(), 1.to_string());
        params.insert("pubtype".to_string(), 1.to_string());
        params.insert("topmenu".to_string(), "1".to_string());
        params.insert("slink".to_owned(), slink);
        let slink_total_number = mobile_bg.total_number(params.clone()).await.unwrap();

        info!("total_number: {}", total_number);
        info!("total_number: {}", slink_total_number);
        assert_eq!(total_number, slink_total_number);

        let number_of_pages = mobile_bg.parent.get_number_of_pages(total_number).unwrap();
        info!("number_of_pages: {}", number_of_pages);
        let mut all = vec![];
        for page in 1..number_of_pages + 1 {
            let ids = mobile_bg
                .get_listed_ids(params.clone(), page)
                .await
                .unwrap();
            assert!(ids.len() > 0);
            info!("ids: {:?}", ids);
            all.extend(ids);
        }
        let unique: HashSet<String> = all.into_iter().collect();
        assert_eq!(unique.len(), total_number as usize);
    }

    #[tokio::test]
    async fn process_mobile_bg_details_test() {
        configure_log4rs(&LOG_CONFIG);
        let mut cars_bg =
            super::MobileBGScraper::new("https://www.mobile.bg/pcgi/mobile.cgi?", 250);
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
        let slink = cars_bg.slink(params.clone()).await.unwrap();
        params.clear();
        params.insert("act".to_owned(), "3".to_owned());
        params.insert("rub".to_string(), 1.to_string());
        params.insert("pubtype".to_string(), 1.to_string());
        params.insert("topmenu".to_string(), "1".to_string());
        params.insert("slink".to_owned(), slink.clone());
        let ids = cars_bg.get_listed_ids(params.clone(), 1).await.unwrap();
        let first = ids.first().unwrap();
        params.clear();
        params.insert("act".to_owned(), "4".to_owned());
        params.insert("topmenu".to_string(), "1".to_string());
        params.insert("submenu".to_string(), "2".to_string());
        params.insert("slink".to_owned(), slink.clone());
        params.insert("adv".to_owned(), first.to_owned());
        let url = cars_bg.parent.search_url(None, params.clone(), 1);
        info!("url: {}", url);
        info!("first: {}", first);
        let details = cars_bg.parse_details(url, first.to_owned()).await.unwrap();
        info!("details: {:?}", details);
        let record = crate::model::records::MobileRecord::from(details);
        info!("record: {:?}", record);
        assert_eq!(record.id, first.to_owned());
    }
}
