use std::{collections::HashMap, str::FromStr};

use log::{error, info};

use crate::{
    model::records::MobileRecord,
    scraper::{
        CarGrScraper::CarGrScraper,
        CarsBgScraper::CarsBGScraper,
        MobileBgScraper::MobileBGScraper,
        ScraperTrait::{LinkId, ScraperTrait},
    },
    services::{
        ScraperService::{process, save},
        Searches::car_gr_new_searches,
    },
    utils::helpers::create_empty_csv,
    CARS_BG_ALL_FILE_NAME, CARS_BG_INSALE_FILE_NAME, CAR_GR_ALL_FILE_NAME, CAR_GR_FILE_NAME,
    MOBILE_BG_ALL_FILE_NAME, MOBILE_BG_FILE_NAME,
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref MOBILE_BG_CRAWLER: MobileBGScraper =
        MobileBGScraper::new("https://www.mobile.bg/pcgi/mobile.cgi?", 250);
    pub static ref CARS_BG_CRAWLER: CarsBGScraper = CarsBGScraper::new("https://www.cars.bg", 250);
    pub static ref CAR_GR_CRAWLER: CarGrScraper = CarGrScraper::new("https://www.car.gr", 250);
}

use super::{
    ScraperService::start,
    Searches::{
        car_gr_all_searches, cars_bg_all_searches, cars_bg_new_searches, mobile_bg_all_searches,
        mobile_bg_new_searches,
    },
};
#[derive(Debug, Clone)]
pub enum Crawlers {
    CarsBG(String),
    MobileBG(String),
    CarGr(String),
}

impl FromStr for Crawlers {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "cars.bg" => Ok(Crawlers::CarsBG(
                r#"https://www.mobile.bg/pcgi/mobile.cgi?"#.to_owned(),
            )),
            "cars_bg" => Ok(Crawlers::CarsBG(
                r#"https://www.mobile.bg/pcgi/mobile.cgi?"#.to_owned(),
            )),
            "cars" => Ok(Crawlers::CarsBG(
                r#"https://www.mobile.bg/pcgi/mobile.cgi?"#.to_owned(),
            )),
            "mobile.bg" => Ok(Crawlers::MobileBG(r#"https://www.cars.bg"#.to_owned())),
            "mobile_bg" => Ok(Crawlers::MobileBG(r#"https://www.cars.bg"#.to_owned())),
            "mobile" => Ok(Crawlers::MobileBG(r#"https://www.cars.bg"#.to_owned())),
            "car.gr" => Ok(Crawlers::CarGr(r#"https://www.car.gr"#.to_owned())),
            _ => Err("Invalid crawler".into()),
        }
    }
}

pub async fn download_all(crawler: &str) -> Result<(), String> {
    let crawler = Crawlers::from_str(crawler)?;
    match crawler {
        Crawlers::CarsBG(_) => {
            let searches = cars_bg_all_searches();
            scrape_all_vehicles(
                CARS_BG_CRAWLER.clone(),
                CARS_BG_ALL_FILE_NAME.to_owned(),
                searches,
            )
            .await
        }
        Crawlers::MobileBG(_) => {
            let searches = mobile_bg_all_searches().await;
            scrape_all_vehicles(
                MOBILE_BG_CRAWLER.clone(),
                MOBILE_BG_ALL_FILE_NAME.to_owned(),
                searches,
            )
            .await
        }
        Crawlers::CarGr(_) => {
            let searches: Vec<HashMap<String, String>> = car_gr_all_searches();
            scrape_all_vehicles(
                MOBILE_BG_CRAWLER.clone(),
                CAR_GR_ALL_FILE_NAME.to_owned(),
                searches,
            )
            .await
        }
    }
}

pub async fn download_new_vehicles(crawler: &str) -> Result<(), String> {
    let crawler = Crawlers::from_str(crawler)?;
    info!("crawler: {:?}", crawler);
    match crawler {
        Crawlers::CarsBG(_) => {
            let searches = cars_bg_new_searches();
            scrape_new_vehicles(
                CARS_BG_CRAWLER.clone(),
                CARS_BG_INSALE_FILE_NAME.to_owned(),
                searches,
            )
            .await
        }
        Crawlers::MobileBG(_) => {
            let searches = mobile_bg_new_searches().await;
            scrape_new_vehicles(
                CAR_GR_CRAWLER.clone(),
                CAR_GR_FILE_NAME.to_owned(),
                searches,
            )
            .await
        }
        Crawlers::CarGr(_) => {
            let searches: Vec<HashMap<String, String>> = car_gr_new_searches();
            info!("searches: {:?}", searches.len());
            scrape_new_vehicles(
                CAR_GR_CRAWLER.clone(),
                CAR_GR_FILE_NAME.to_owned(),
                searches,
            )
            .await
        }
    }
}

pub async fn scrape_new_vehicles<T: ScraperTrait + Clone + Send>(
    scraper: T,
    file_name: String,
    searches: Vec<HashMap<String, String>>,
) -> Result<(), String>
where
    T: 'static,
{
    let (mut link_producer, mut link_receiver) = tokio::sync::mpsc::channel::<LinkId>(250);
    let (mut record_producer, record_receiver) = tokio::sync::mpsc::channel::<MobileRecord>(250);
    if create_empty_csv::<MobileRecord>(&file_name).is_err() {
        error!("Failed to create file {}", file_name.clone());
    }
    let process_scraper = scraper.clone();
    let headers = scraper.clone().headers().await;
    let start_handler = tokio::spawn(async move {
        start(
            Box::new(scraper),
            searches,
            &mut link_producer,
            headers.clone(),
        )
        .await
    });
    let process_handler = tokio::spawn(async move {
        process(
            process_scraper,
            &mut link_receiver,
            &mut record_producer,
            HashMap::new(),
        )
        .await
    });
    let save_to_file = tokio::spawn(async move { save(record_receiver, file_name).await });

    if let (Ok(_), Ok(_), Ok(_)) = tokio::join!(start_handler, process_handler, save_to_file) {
        info!("All tasks completed successfully");
        Ok(())
    } else {
        error!("One or more tasks failed");
        Err("One or more tasks failed".into())
    }
}

pub async fn scrape_all_vehicles<T: ScraperTrait + Clone + Send>(
    scraper: T,
    file_name: String,
    searches: Vec<HashMap<String, String>>,
) -> Result<(), String>
where
    T: 'static,
{
    let (mut producer, receiver) = tokio::sync::mpsc::channel::<LinkId>(250);
    if create_empty_csv::<MobileRecord>(&file_name).is_err() {
        error!("Failed to create file {}", file_name.clone());
    }
    let start_handler = tokio::spawn(async move {
        start(Box::new(scraper), searches, &mut producer, HashMap::new()).await
    });

    let save_to_file = tokio::spawn(async move { save(receiver, file_name).await });

    if let (Ok(_), Ok(_)) = tokio::join!(start_handler, save_to_file) {
        info!("All tasks completed successfully");
        Ok(())
    } else {
        error!("One or more tasks failed");
        Err("One or more tasks failed".into())
    }
}

#[cfg(test)]
mod app_test {
    use std::collections::HashMap;

    use log::info;

    use super::scrape_new_vehicles;
    use crate::scraper::CarsBgScraper::CarsBGScraper;
    use crate::scraper::MobileBgScraper::MobileBGScraper;
    use crate::scraper::ScraperTrait::ScraperTrait;
    use crate::services::ScraperAppService::{CARS_BG_CRAWLER, MOBILE_BG_CRAWLER};
    use crate::services::Searches::{cars_bg_new_searches, mobile_bg_new_searches};
    use crate::utils::helpers::configure_log4rs;
    use crate::LOG_CONFIG;

    #[tokio::test]
    async fn test_run_carsbg_scraper() {
        configure_log4rs(&LOG_CONFIG);
        let mut params = HashMap::new();
        //subm=1&add_search=1&typeoffer=1&priceFrom=18000&priceTo=30000&yearFrom=2007&yearTo=2011&page=32
        params.insert("subm".to_owned(), "1".to_owned());
        params.insert("add_search".to_owned(), "1".to_owned());
        params.insert("typeoffer".to_owned(), "1".to_owned());
        params.insert("priceFrom".to_owned(), "29500".to_owned());
        params.insert("priceTo".to_owned(), "30000".to_owned());
        params.insert("yearFrom".to_owned(), "2010".to_owned());
        params.insert("yearTo".to_owned(), "2011".to_owned());
        let searches = vec![params];
        let scraper = CarsBGScraper::new("https://www.cars.bg", 250);
        scrape_new_vehicles(
            scraper,
            "./resources/test-data/test.csv".to_owned(),
            searches,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_run_mobilebg_scraper() {
        configure_log4rs(&LOG_CONFIG);
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

        let scraper = MobileBGScraper::new("https://www.mobile.bg/pcgi/mobile.cgi?", 250);
        let slink = scraper.slink(params.clone()).await.unwrap();
        params.clear();
        params.insert("act".to_owned(), "3".to_owned());
        params.insert("rub".to_string(), 1.to_string());
        params.insert("pubtype".to_string(), 1.to_string());
        params.insert("topmenu".to_string(), "1".to_string());
        params.insert("slink".to_owned(), slink.clone());
        let searches = vec![params];
        scrape_new_vehicles(
            scraper,
            "./resources/test-data/test.csv".to_owned(),
            searches,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_cars_bg_searches() {
        configure_log4rs(&LOG_CONFIG);
        let searches = cars_bg_new_searches();
        let mut total = 0;
        for search in searches {
            let total_number = CARS_BG_CRAWLER
                .total_number(search.clone(), HashMap::new())
                .await
                .unwrap();
            total += total_number;
            info!("total_number: {} for search: {:?}", total_number, search);
        }
        info!("total: {}", total);
    }

    #[tokio::test]
    async fn test_mobile_bg_searches() {
        configure_log4rs(&LOG_CONFIG);
        let searches = mobile_bg_new_searches().await;
        let mut total = 0;
        for search in searches {
            let total_number = MOBILE_BG_CRAWLER
                .total_number(search.clone(), HashMap::new())
                .await
                .unwrap();
            total += total_number;
            info!("total_number: {} for search: {:?}", total_number, search);
        }
        info!("total: {}", total);
    }
}
