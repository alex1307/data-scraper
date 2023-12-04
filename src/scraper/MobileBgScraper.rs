use std::{collections::HashMap, time::Duration};

use async_trait::async_trait;

use log::info;
use regex::Regex;
use reqwest::header;
use scraper::{Html, Selector};

use super::{
    mobile_bg_helpers::{details2map, get_url},
    ScraperTrait::{LinkId, Scraper, ScraperTrait},
};
use crate::{scraper::mobile_bg_helpers::slink, BROWSER_USER_AGENT};
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

    pub fn slink(&self, html: &str) -> Result<String, String> {
        let slink = slink(html);
        if slink.is_empty() {
            return Err("slink not found".to_string());
        }
        Ok(slink)
    }
}

#[async_trait]
impl ScraperTrait for MobileBGScraper {
    async fn get_html(
        &self,
        path: Option<String>,
        params: HashMap<String, String>,
        page: u32,
    ) -> Result<String, String> {
        let url = self.parent.search_url(path, params, page);
        self.parent
            .html_search(&url, Some("windows-1251".to_string()))
            .await
    }

    fn total_number(&self, html: &str) -> Result<u32, String> {
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
    ) -> Result<Vec<LinkId>, String> {
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
                let url = if url.contains(r#"https:"#) {
                    url
                } else {
                    format!("https:{}", url)
                };
                if let Some(caps) = re.captures(&url) {
                    if let Some(matched) = caps.get(1) {
                        links.push(LinkId {
                            id: matched.as_str().to_owned(),
                            url,
                        });
                    }
                }
            }
        }
        Ok(links)
    }

    async fn parse_details(&self, link: LinkId) -> Result<HashMap<String, String>, String> {
        let html = self
            .parent
            .html_search(&link.url, Some("windows-1251".to_string()))
            .await?;
        let document = Html::parse_document(&html);
        let mut result = details2map(document);
        result.insert("id".to_owned(), link.id);
        Ok(result)
    }
    fn get_number_of_pages(&self, total_number: u32) -> Result<u32, String> {
        self.parent.get_number_of_pages(total_number)
    }
}

#[cfg(test)]
mod screaper_mobile_bg_test {
    use std::collections::{HashMap, HashSet};

    use crate::{
        scraper::{
            MobileBgScraper,
            ScraperTrait::{LinkId, ScraperTrait as _},
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

        let html = mobile_bg.get_html(None, params.clone(), 1).await.unwrap();
        let total_number = mobile_bg.total_number(&html).unwrap();
        let slink = mobile_bg.slink(&html).await.unwrap();
        params.clear();
        params.insert("act".to_owned(), "3".to_owned());
        params.insert("rub".to_string(), 1.to_string());
        params.insert("pubtype".to_string(), 1.to_string());
        params.insert("topmenu".to_string(), "1".to_string());
        params.insert("slink".to_owned(), slink);

        let html = mobile_bg.get_html(None, params.clone(), 1).await.unwrap();
        let slink_totals = mobile_bg.total_number(&html).unwrap();
        let slink = mobile_bg.slink(&html).await.unwrap();

        info!("total_number: {}", total_number);
        info!("total_number: {}", slink_totals);
        assert_eq!(total_number, slink_totals);

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
        let unique: HashSet<LinkId> = all.into_iter().collect();
        assert_eq!(unique.len(), total_number as usize);
    }

    #[tokio::test]
    async fn process_mobile_bg_details_test() {
        configure_log4rs(&LOG_CONFIG);
        let mobile_bg: MobileBgScraper::MobileBGScraper =
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
        let html = mobile_bg.get_html(None, params, 1).await.unwrap();
        let slink_totals = mobile_bg.total_number(&html).unwrap();
        let slink = mobile_bg.slink(&html).await.unwrap();

        params.clear();
        params.insert("act".to_owned(), "3".to_owned());
        params.insert("rub".to_string(), 1.to_string());
        params.insert("pubtype".to_string(), 1.to_string());
        params.insert("topmenu".to_string(), "1".to_string());
        params.insert("slink".to_owned(), slink.clone());

        let html = mobile_bg.get_html(None, params, 1).await.unwrap();
        let ids = mobile_bg.get_listed_ids(params.clone(), 1).await.unwrap();
        let first = ids.first().unwrap();
        params.clear();
        params.insert("act".to_owned(), "4".to_owned());
        params.insert("topmenu".to_string(), "1".to_string());
        params.insert("submenu".to_string(), "2".to_string());
        params.insert("slink".to_owned(), slink.clone());
        params.insert("adv".to_owned(), first.id.to_owned());
        let url = mobile_bg.parent.search_url(None, params.clone(), 1);
        info!("url: {}", url);
        info!("first: {:?}", first);
        let details = mobile_bg.parse_details(first.clone()).await.unwrap();
        info!("details: {:?}", details);
        let record = crate::model::records::MobileRecord::from(details);
        info!("record: {:?}", record);
        assert_eq!(record.id, first.id);
    }
}
