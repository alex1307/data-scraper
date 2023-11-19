use std::{collections::HashMap, time::Duration};

use async_trait::async_trait;
use encoding_rs::{Encoding, UTF_8};
use lazy_static::lazy_static;
use log::info;
use regex::Regex;
use scraper::{Html, Selector};
use serde::Deserialize;

use crate::{scraper::{cars_bg::read_carsbg_details, mobile_bg::slink}, BROWSER_USER_AGENT};

use super::mobile_bg::{get_url, details2map};


lazy_static! {
    pub static ref REQWEST_ASYNC_CLIENT: reqwest::Client = reqwest::Client::builder()
        .user_agent(BROWSER_USER_AGENT)
        .build()
        .unwrap();
}

#[derive(Debug, Clone)]
pub struct Scraper {
    url: String,
    page: String,
    wait_time_ms: u64,
}
#[async_trait]
pub trait ScraperTrait {
    async fn total_number(&self, params: HashMap<String, String>) -> Result<u32, String>;
    async fn get_listed_ids(
        &self,
        params: HashMap<String, String>,
        page_number: u32,
    ) -> Result<Vec<String>, String>;
    async fn parse_details(
        &self,
        url: String,
        id: String,
    ) -> Result<HashMap<String, String>, String>;
}
#[derive(Debug, Clone, Deserialize)]
struct ViewCountsCarsBG {
    status: String,
    value_resettable: u32,
}

impl Scraper {
    pub fn new(url: &str, page: String, wait_time_ms: u64) -> Self {
        Scraper {
            url: url.to_string(),
            page,
            wait_time_ms,
        }
    }

    pub fn search_url(
        &self,
        path: Option<String>,
        params: HashMap<String, String>,
        page: u32,
    ) -> String {
        let mut url = if let Some(path) = path {
            format!("{}{}", self.url, path)
        } else {
            self.url.clone()
        };

        if params.is_empty() {
            return url;
        }

        for (key, value) in params.iter() {
            url = format!("{}{}={}&", url, key, value);
        }
        if page == 0 {
            return url.trim_end_matches('&').to_owned();
        }

        url = format!("{}{}={}", url, self.page, page);
        url
    }

    pub fn get_number_of_pages(&self, total_number: u32) -> Result<u32, String> {
        if total_number == 0 {
            return Ok(0);
        }
        let number_of_pages: u32 = (total_number / 20)
            .try_into()
            .map_err(|_| "Failed to convert total number of pages to u32")?;
        if total_number % 20 == 0 {
            return Ok(number_of_pages);
        } else {
            return Ok(number_of_pages + 1);
        }
    }

    async fn html_search(
        &self,
        url: &str,
        decoding_from: Option<String>,
    ) -> Result<String, String> {
        let response = REQWEST_ASYNC_CLIENT
            .get(url)
            .send()
            .await;
        if let Ok(response) = response {
            if let Some(label) = decoding_from {
                let bytes = response.bytes().await.unwrap().to_vec();
                //"windows-1251"
                let encoding = Encoding::for_label(label.as_bytes()).unwrap();
                let (decoded, _, _) = encoding.decode(&bytes);
                let utf8_html = UTF_8.encode(&decoded).0;
                let response = String::from_utf8_lossy(&utf8_html);
                return Ok(response.to_string());
            } else if let Ok(html) = response.text().await {
                return Ok(html);    
            }
        }
        Err(format!("Failed to get html from {}", url))
    }
}

pub async fn get_view_count(id: String) -> Result<u32, String> {
    let url = format!("https://stats.cars.bg/add/?object_id={}", id);
    REQWEST_ASYNC_CLIENT.get(url).send().await.unwrap();
    let url = format!("https://stats.cars.bg/get/?object_id={}", id);
    let response = REQWEST_ASYNC_CLIENT.get(url).send().await;

    match response {
        Ok(response) => match response.json::<ViewCountsCarsBG>().await {
            Ok(views) => {
                return Ok(views.value_resettable);
            }
            Err(e) => return Err(e.to_string()),
        },
        Err(e) => Err(e.to_string()),
    }
}

pub struct CarsBGScraper {
    pub parent: Scraper,
}

pub struct MobileBGScraper {
    pub parent: Scraper,
    slink: Option<String>,
}

impl CarsBGScraper {
    pub fn new(url: &str, wait_time_ms: u64) -> Self {
        CarsBGScraper {
            parent: Scraper::new(url, "page".to_string(), wait_time_ms),
        }
    }
}

impl MobileBGScraper {
    pub fn new(url: &str, wait_time_ms: u64) -> Self {
        MobileBGScraper {
            parent: Scraper::new(url, "f1".to_string(), wait_time_ms),
            slink: None,
        }
    }

    pub async fn slink(&mut self, params: HashMap<String, String>) -> Result<String, String> {
        let url = self
            .parent
            .search_url(None, params.clone(), 0);
        let html = self.parent.html_search(url.as_str(), Some("windows-1251".to_string())).await?;
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
    ) -> Result<Vec<String>, String> {
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
                let href = e.value().attr("href").unwrap();
                let id = href.split("/offer/").last().unwrap();
                ids.push(id.to_owned());
                break;
            }
        }
        info!("ids: {:?}", ids);
        Ok(ids)
    }

    async fn parse_details(
        &self,
        url: String,
        id: String,
    ) -> Result<HashMap<String, String>, String> {
        let html = self.parent.html_search(&url, None).await?;
        let mut result = read_carsbg_details(html);
        let views = get_view_count(id.clone()).await?;
        result.insert("id".to_owned(), id);
        result.insert("view_count".to_owned(), views.to_string());
        Ok(result)
    }
}

#[async_trait]
impl ScraperTrait for MobileBGScraper {
    async fn total_number(&self, params: HashMap<String, String>) -> Result<u32, String> {
        let url = self
            .parent
            .search_url(None, params.clone(), 0);
        let html = self.parent.html_search(url.as_str(), Some("windows-1251".to_string())).await?;
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
        let url = self
            .parent
            .search_url(None, params.clone(), page_number);
        let html = self.parent.html_search(&url,
            Some("windows-1251".to_string()),
        ).await?;
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
        let html = self.parent.html_search(&url, Some("windows-1251".to_string())).await?;
        let document = Html::parse_document(&html);
        let mut result = details2map(document);
        result.insert("id".to_owned(), id);
        Ok(result)
    }
}

#[cfg(test)]
mod cars_bg_tests {
    use std::collections::HashMap;

    use log::info;

    use crate::{
        scraper::scrapers::ScraperTrait as _, utils::helpers::configure_log4rs, LOG_CONFIG,
    };

    #[tokio::test]
    async fn total_number_test() {
        configure_log4rs(&LOG_CONFIG);
        let cars_bg = super::CarsBGScraper::new("https://www.cars.bg", 250);
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
    }

    #[tokio::test]
    async fn read_details_test() {
        configure_log4rs(&LOG_CONFIG);
        let cars_bg = super::CarsBGScraper::new("https://www.cars.bg", 250);
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
        let path = Some(format!("/offer/{}", first));
        let search_url = cars_bg.parent.search_url(path, HashMap::new(), 0);
        info!("search_url: {}", search_url);
        let details = cars_bg
            .parse_details(search_url, first.to_owned())
            .await
            .unwrap();
        assert!(details.len() > 0);
        assert_eq!(details.get("id").unwrap(), first);
        info!("details: {:?}", details);
        let record = crate::model::records::MobileRecord::from(details);
        info!("record: {:?}", record);
    }
}

#[cfg(test)]
mod mobile_bg_test{
    use std::collections::{HashMap, HashSet};

    use crate::{utils::helpers::configure_log4rs, LOG_CONFIG, scraper::scrapers::ScraperTrait};
    use log::info;

    #[tokio::test]
    async fn total_number_test() {
        configure_log4rs(&LOG_CONFIG);
        let mut mobile_bg = super::MobileBGScraper::new("https://www.mobile.bg/pcgi/mobile.cgi?", 250);
        let mut params = HashMap::new();
        params.insert("act".to_owned(), "3".to_owned());
        params.insert("f10".to_owned(), "2004".to_owned());
        params.insert("topmenu".to_string(), "1".to_string());
        params.insert("rub".to_string(), 1.to_string());
        params.insert("pubtype".to_string(), 1.to_string());
        params.insert("f7".to_string(), 10000.to_string());
        params.insert("f94".to_string(), "1~%CA%E0%EF%E0%F0%E8%F0%E0%ED%5C%CF%F0%EE%E4%E0%E4%E5%ED".to_string());
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
            let ids = mobile_bg.get_listed_ids(params.clone(), page).await.unwrap();
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
        let mut cars_bg = super::MobileBGScraper::new("https://www.mobile.bg/pcgi/mobile.cgi?", 250);
        let mut params = HashMap::new();
        params.insert("act".to_owned(), "3".to_owned());
        params.insert("f10".to_owned(), "2004".to_owned());
        params.insert("topmenu".to_string(), "1".to_string());
        params.insert("rub".to_string(), 1.to_string());
        params.insert("pubtype".to_string(), 1.to_string());
        params.insert("f7".to_string(), 10000.to_string());
        params.insert("f94".to_string(), "1~%CA%E0%EF%E0%F0%E8%F0%E0%ED%5C%CF%F0%EE%E4%E0%E4%E5%ED".to_string());
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
