use std::{collections::HashMap, str::FromStr, time::Duration};

use async_trait::async_trait;
use regex::Regex;
use scraper::{Html, Selector};

use super::Traits::{ScrapeListTrait, Scraper, ScraperTrait};
use crate::{
    helpers::MobileBgHTMLHelper::{resume_info, slink},
    model::{
        enums::{Engine, Gearbox},
        records::MobileRecord,
        VehicleDataModel::ScrapedListData,
    },
    services::Searches::to_slink,
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

    pub fn slink(&self, html: &str) -> Result<String, String> {
        let slink = slink(html);
        if slink.is_empty() {
            return Err("slink not found".to_string());
        }
        Ok(slink)
    }
}

#[async_trait]
impl ScrapeListTrait<MobileRecord> for MobileBGScraper {
    async fn process_listed_results(
        &self,
        params: HashMap<String, String>,
        page_number: u32,
    ) -> Result<ScrapedListData<MobileRecord>, String> {
        let mut search = params.clone();
        to_slink(&mut search).await;
        let url = self.parent.search_url(None, search, page_number);
        let html = self
            .parent
            .html_search(&url, Some("windows-1251".to_string()))
            .await?;
        let value = params.get("gearbox").unwrap().to_string();
        let gearbox = Gearbox::from_str(&value).unwrap();
        let value = params.get("engine").unwrap().to_string();
        let engine = Engine::from_str(&value).unwrap();
        let power: u32 = params.get("power").unwrap().parse().unwrap();
        let seller = params.get("seller").unwrap();
        let vehicles = resume_info(html.as_str(), gearbox, engine, power, "Dealer" == seller);

        Ok(ScrapedListData::Values(vehicles))
    }
}

// #[async_trait]
// impl RequestResponseTrait<LinkId, MobileRecord> for MobileBGScraper {
//     async fn handle_request(&self, link: LinkId) -> Result<MobileRecord, String> {
//         let html = self
//             .parent
//             .html_search(&link.url, Some("windows-1251".to_string()))
//             .await?;
//         let document = Html::parse_document(&html);
//         let mut result = details2map(document);
//         result.insert("id".to_owned(), link.id.clone());
//         if result.get(PRICE_KEY.to_string().as_str()).is_none() {
//             Err(format!("invalid/incompete price data for: {}", &link.id))
//         } else if result.get(MAKE_KEY.to_string().as_str()).is_none() {
//             Err(format!("invalid/incompete make data for: {}", &link.id))
//         } else if result.get(YEAR_KEY.to_string().as_str()).is_none() {
//             Err(format!("invalid/incompete year data for: {}", &link.id))
//         } else if result.get(MILEAGE_KEY.to_string().as_str()).is_none() {
//             Err(format!("invalid/incompete mileage data for: {}", &link.id))
//         } else if result.get(ENGINE_KEY.to_string().as_str()).is_none() {
//             Err(format!("invalid/incompete engine data for: {}", &link.id))
//         } else if result.get(GEARBOX_KEY.to_string().as_str()).is_none() {
//             Err(format!("invalid/incompete gearbox data for: {}", &link.id))
//         } else {
//             let record = MobileRecord::from(result);
//             Ok(record)
//         }
//     }
// }

#[async_trait]
impl ScraperTrait for MobileBGScraper {
    async fn get_html(&self, params: HashMap<String, String>, page: u32) -> Result<String, String> {
        let url = self.parent.search_url(self.get_search_path(), params, page);
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
        let slink = mobile_bg.slink(&html).unwrap();
        params.clear();
        params.insert("act".to_owned(), "3".to_owned());
        params.insert("rub".to_string(), 1.to_string());
        params.insert("pubtype".to_string(), 1.to_string());
        params.insert("topmenu".to_string(), "1".to_string());
        params.insert("slink".to_owned(), slink);

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

    // #[tokio::test]
    // async fn process_mobile_bg_details_test() {
    //     configure_log4rs(&LOG_CONFIG);
    //     let mobile_bg: MobileBgScraper::MobileBGScraper =
    //         MobileBgScraper::MobileBGScraper::new("https://www.mobile.bg/pcgi/mobile.cgi?", 250);
    //     let mut params = HashMap::new();
    //     params.insert("act".to_owned(), "3".to_owned());
    //     params.insert("f10".to_owned(), "2004".to_owned());
    //     params.insert("topmenu".to_string(), "1".to_string());
    //     params.insert("rub".to_string(), 1.to_string());
    //     params.insert("pubtype".to_string(), 1.to_string());
    //     params.insert("f7".to_string(), 10000.to_string());
    //     params.insert(
    //         "f94".to_string(),
    //         "1~%CA%E0%EF%E0%F0%E8%F0%E0%ED%5C%CF%F0%EE%E4%E0%E4%E5%ED".to_string(),
    //     );
    //     let html = mobile_bg.get_html(params.clone(), 1).await.unwrap();
    //     let slink = mobile_bg.slink(&html).unwrap();

    //     params.clear();
    //     params.insert("act".to_owned(), "3".to_owned());
    //     params.insert("rub".to_string(), 1.to_string());
    //     params.insert("pubtype".to_string(), 1.to_string());
    //     params.insert("topmenu".to_string(), "1".to_string());
    //     params.insert("slink".to_owned(), slink.clone());

    //     let data = mobile_bg
    //         .process_listed_results(params.clone(), 1)
    //         .await
    //         .unwrap();
    //     let mut id: LinkId = LinkId {
    //         id: "".to_owned(),
    //         source: "mobile.bg".to_owned(),
    //         url: "".to_owned(),
    //     };
    //     match data {
    //         ScrapedListData::Values(ids) => {
    //             assert!(ids.len() > 0);
    //             info!("ids: {:?}", ids);
    //             id = ids[0].clone();
    //         }
    //         ScrapedListData::Error(error) => {
    //             info!("error: {}", error);
    //         }
    //         ScrapedListData::SingleValue(link) => {
    //             info!("link: {:?}", link);
    //         }
    //     }

    //     params.clear();
    //     params.insert("act".to_owned(), "4".to_owned());
    //     params.insert("topmenu".to_string(), "1".to_string());
    //     params.insert("submenu".to_string(), "2".to_string());
    //     params.insert("slink".to_owned(), slink.clone());
    //     params.insert("adv".to_owned(), id.id.to_owned());
    //     let url = mobile_bg.parent.search_url(None, params.clone(), 1);
    //     info!("url: {}", url);
    //     let details = mobile_bg.handle_request(id.clone()).await.unwrap();
    //     info!("details: {:?}", details);
    //     let record = crate::model::records::MobileRecord::from(details);
    //     info!("record: {:?}", record);
    //     assert_eq!(record.id, id.id);
    // }
}
