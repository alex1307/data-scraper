use std::{collections::HashMap, str::FromStr, time::Duration};

use async_trait::async_trait;

use rand::Rng;
use regex::Regex;
use scraper::{Html, Selector};
use tokio::time::sleep;

use super::Traits::{ScrapeListTrait, Scraper, ScraperTrait};
use crate::{
    helpers::MobileBgHTMLHelper::process_listing,
    model::{
        enums::{Engine, Gearbox},
        VehicleDataModel::ScrapedListData,
        VehicleRecord::MobileRecord,
    },
    services::SearchBuilder::{
        MOBILE_BG_POWER_FROM, MOBILE_BG_POWER_TO, MOBILE_BG_YEARS_FROM, MOBILE_BG_YEARS_TO,
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

    fn search_url(&self, params: HashMap<String, String>, page: u32) -> String {
        let url = "https://www.mobile.bg/obiavi/avtomobili-dzhipove/{engine}/{gearbox}{page}/ot-{yearFrom}/do-{yearTo}{page}?f24=2&&engine_power={powerFrom}&engine_power1={powerTo}{priceFrom}{priceTo}";
        let url = if let Some(from) = params.get("priceFrom") {
            url.replace("{priceFrom}", format!("&price={}", from).as_str())
        } else {
            url.replace("{priceFrom}", "")
        };
        let url = if let Some(to) = params.get("priceTo") {
            url.replace("{priceTo}", format!("&price1={}", to).as_str())
        } else {
            url.replace("{priceTo}", "")
        };

        let url = if let Some(powerTo) = params.get(MOBILE_BG_POWER_TO) {
            url.replace("{powerTo}", powerTo)
        } else {
            url.replace("&engine_power1={powerTo}", "")
        };

        let fromYear = params.get(MOBILE_BG_YEARS_FROM).unwrap();
        let toYear = params.get(MOBILE_BG_YEARS_TO).unwrap();
        let fromPower = params.get(MOBILE_BG_POWER_FROM).unwrap();

        let engine = params.get("engine_url").unwrap();
        let gearbox = params.get("gearbox_url").unwrap();

        let url = url.replace("{yearFrom}", fromYear);
        let url = url.replace("{yearTo}", toYear);
        let url = url.replace("{powerFrom}", fromPower);
        let url = url.replace("{engine}", engine);
        let url = url.replace("{gearbox}", gearbox);
        let url = if page > 1 {
            url.replace("{page}", &format!("/p-{}", page))
        } else {
            url.replace("{page}", "")
        };
        url
    }
}

#[async_trait]
impl ScrapeListTrait<MobileRecord> for MobileBGScraper {
    async fn process_listed_results(
        &self,
        params: HashMap<String, String>,
        page_number: u32,
    ) -> Result<ScrapedListData<MobileRecord>, String> {
        let search = params.clone();
        let url = self.search_url(search, page_number);
        let html = self
            .parent
            .html_search(&url, Some("windows-1251".to_string()))
            .await?;

        let value = params.get("gearbox").unwrap().to_string();
        let gearbox = Gearbox::from_str(&value).unwrap();
        let value = params.get("engine").unwrap().to_string();
        let engine = Engine::from_str(&value).unwrap();
        let power: u32 = params.get("power").unwrap().parse().unwrap();
        let vehicles = process_listing(html.as_str(), gearbox, engine, power);
        if vehicles.is_empty() {
            panic!("{}", html);
        }
        let waiting_time_ms: u64 = rand::thread_rng().gen_range(1_000..3_000);
        sleep(Duration::from_millis(waiting_time_ms as u64)).await;

        Ok(ScrapedListData::Values(vehicles))
    }
}

#[async_trait]
impl ScraperTrait for MobileBGScraper {
    async fn get_html(&self, params: HashMap<String, String>, page: u32) -> Result<String, String> {
        let url = self.search_url(params, page);
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
}

#[cfg(test)]
mod screaper_mobile_bg_test {
    use std::collections::HashMap;

    use crate::{
        model::VehicleDataModel::ScrapedListData,
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

        let html = mobile_bg.get_html(params.clone(), 1).await.unwrap();
        let total_number = mobile_bg.total_number(&html).unwrap();
        params.clear();
        params.insert("act".to_owned(), "3".to_owned());
        params.insert("rub".to_string(), 1.to_string());
        params.insert("pubtype".to_string(), 1.to_string());
        params.insert("topmenu".to_string(), "1".to_string());

        let html = mobile_bg.get_html(params.clone(), 1).await.unwrap();
        let slink_totals = mobile_bg.total_number(&html).unwrap();

        assert_eq!(total_number, slink_totals);

        let number_of_pages = mobile_bg.parent.get_number_of_pages(total_number).unwrap();
        let mut all = vec![];
        for page in 1..number_of_pages + 1 {
            let data = mobile_bg
                .process_listed_results(params.clone(), page)
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
