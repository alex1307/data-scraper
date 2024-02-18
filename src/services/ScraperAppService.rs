use std::{collections::HashMap, fmt::Debug, str::FromStr};

use log::{error, info};

use crate::{
    model::{records::MobileRecord, AutoUncleVehicle::AutoUncleVehicle, VehicleDataModel::LinkId},
    scraper::{
        AutouncleScraper::AutouncleScraper,
        CarGrScraper::CarGrScraper,
        CarsBgScraper::CarsBGScraper,
        MobileBgScraper::MobileBGScraper,
        Traits::{RequestResponseTrait, ScrapeListTrait, ScraperTrait},
    },
    services::ScraperService::process_details,
    AUTOUNCLE_ALL_SEARCHES_LOG, CARS_BG_ALL_SEARCHES_LOG, CARS_BG_NEW_SEARCHES_LOG,
    MOBILE_BG_ALL_SEARCHES_LOG, MOBILE_BG_NEW_SEARCHES_LOG,
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref MOBILE_BG_CRAWLER: MobileBGScraper =
        MobileBGScraper::new("https://www.mobile.bg/pcgi/mobile.cgi?", 250);
    pub static ref CARS_BG_CRAWLER: CarsBGScraper = CarsBGScraper::new("https://www.cars.bg", 250);
    pub static ref CAR_GR_CRAWLER: CarGrScraper = CarGrScraper::new("https://www.car.gr", 250);
    pub static ref AUTOUNCLE_CRAWLER: AutouncleScraper =
        AutouncleScraper::new("https://www.autouncle.ro/en/cars_search?", 250);
}

use super::{
    ScraperService::{
        log_search, process_list, process_list_and_send, send_autonucle_kafka, send_links_to_kafka,
        send_mobile_record_to_kafka,
    },
    Searches::{
        autouncle_all_searches, cars_bg_all_searches, cars_bg_new_searches, mobile_bg_all_searches,
        mobile_bg_new_searches, to_slink_searches,
    },
};
#[derive(Debug, Clone)]
pub enum Crawlers {
    CarsBG(String),
    MobileBG(String),
    Autouncle(String),
}

impl FromStr for Crawlers {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "cars.bg" => Ok(Crawlers::CarsBG(r#"https://www.cars.bg"#.to_owned())),
            "cars_bg" => Ok(Crawlers::CarsBG(r#"https://www.cars.bg"#.to_owned())),
            "cars" => Ok(Crawlers::CarsBG(r#"https://www.cars.bg"#.to_owned())),
            "mobile.bg" => Ok(Crawlers::MobileBG(r#"https://www.cars.bg"#.to_owned())),
            "mobile_bg" => Ok(Crawlers::MobileBG(r#"https://www.cars.bg"#.to_owned())),
            "mobile" => Ok(Crawlers::MobileBG(r#"https://www.cars.bg"#.to_owned())),
            "autouncle" => Ok(Crawlers::Autouncle(
                r#"https://www.autouncle.ro"#.to_owned(),
            )),
            "autouncle.ro" => Ok(Crawlers::Autouncle(
                r#"https://www.autouncle.ro"#.to_owned(),
            )),
            _ => Err("Invalid crawler".into()),
        }
    }
}

pub async fn download_all(crawler: &str) -> Result<(), String> {
    let crawler = Crawlers::from_str(crawler)?;
    info!("Starting crawler (all): {:?}", crawler);
    match crawler {
        Crawlers::CarsBG(_) => {
            let searches = cars_bg_all_searches();
            download_list_data(
                CARS_BG_CRAWLER.clone(),
                CARS_BG_ALL_SEARCHES_LOG.to_owned(),
                searches,
            )
            .await
        }
        Crawlers::MobileBG(_) => {
            info!("Starting mobile.bg");
            let searches = mobile_bg_all_searches();
            download_list_data(
                MOBILE_BG_CRAWLER.clone(),
                MOBILE_BG_ALL_SEARCHES_LOG.to_owned(),
                searches,
            )
            .await
        }
        Crawlers::Autouncle(_) => {
            let searches: Vec<HashMap<String, String>> = autouncle_all_searches();
            download_autouncle_data(AUTOUNCLE_CRAWLER.clone(), searches).await
        }
    }
}

pub async fn download_details<S>(
    scraper: S,
    file_search_name: String,
    searches: Vec<HashMap<String, String>>,
) -> Result<(), String>
where
    S: ScraperTrait
        + ScrapeListTrait<LinkId>
        + RequestResponseTrait<LinkId, MobileRecord>
        + Clone
        + Send
        + 'static,
{
    let (mut link_producer, mut link_receiver) = tokio::sync::mpsc::channel::<LinkId>(250);
    let (mut record_producer, mut record_receiver) =
        tokio::sync::mpsc::channel::<MobileRecord>(250);
    let (mut search_producer, search_receiver) =
        tokio::sync::mpsc::channel::<HashMap<String, String>>(250);
    let process_scraper = scraper.clone();
    let start_handler = tokio::spawn(async move {
        process_list(
            Box::new(scraper),
            searches,
            &mut link_producer,
            &mut search_producer,
        )
        .await
    });
    let process_handler = tokio::spawn(async move {
        process_details(process_scraper, &mut link_receiver, &mut record_producer).await
    });
    let kafka_handler =
        tokio::spawn(async move { send_mobile_record_to_kafka(&mut record_receiver).await });
    let save_searches =
        tokio::spawn(async move { log_search(search_receiver, file_search_name).await });

    if let (Ok(_), Ok(_), Ok(_), Ok(_)) =
        tokio::join!(start_handler, process_handler, kafka_handler, save_searches)
    {
        info!("All tasks completed successfully");
        Ok(())
    } else {
        error!("One or more tasks failed");
        Err("One or more tasks failed".into())
    }
}

pub async fn download_autouncle_data<S>(
    scraper: S,
    searches: Vec<HashMap<String, String>>, // Same issue with U
) -> Result<(), String>
where
    S: ScraperTrait + ScrapeListTrait<AutoUncleVehicle> + Clone + Send + 'static,
{
    let (mut data_producer, mut data_receiver) =
        tokio::sync::mpsc::channel::<AutoUncleVehicle>(1000);

    let (search_producer, search_receiver) =
        tokio::sync::mpsc::channel::<HashMap<String, String>>(250);

    let start_handler = tokio::spawn(async move {
        process_list_and_send(
            Box::new(scraper),
            searches,
            &mut data_producer,
            search_producer,
        )
        .await
    });
    let kafka_handler = tokio::spawn(async move { send_autonucle_kafka(&mut data_receiver).await });

    let save_searches = tokio::spawn(async move {
        log_search(search_receiver, AUTOUNCLE_ALL_SEARCHES_LOG.to_owned()).await
    });
    if let (Ok(_), Ok(_), Ok(_)) = tokio::join!(start_handler, kafka_handler, save_searches) {
        info!("All tasks completed successfully");
        Ok(())
    } else {
        error!("One or more tasks failed");
        Err("One or more tasks failed".into())
    }
}

pub async fn download_carsbg_data<S>(
    scraper: S,
    searches: Vec<HashMap<String, String>>, // Same issue with U
) -> Result<(), String>
where
    S: ScraperTrait + ScrapeListTrait<MobileRecord> + Clone + Send + 'static,
{
    let (mut data_producer, mut data_receiver) = tokio::sync::mpsc::channel::<MobileRecord>(1000);

    let (search_producer, search_receiver) =
        tokio::sync::mpsc::channel::<HashMap<String, String>>(250);

    let start_handler = tokio::spawn(async move {
        process_list_and_send(
            Box::new(scraper),
            searches,
            &mut data_producer,
            search_producer,
        )
        .await
    });
    let kafka_handler =
        tokio::spawn(async move { send_mobile_record_to_kafka(&mut data_receiver).await });

    let save_searches = tokio::spawn(async move {
        log_search(search_receiver, AUTOUNCLE_ALL_SEARCHES_LOG.to_owned()).await
    });
    if let (Ok(_), Ok(_), Ok(_)) = tokio::join!(start_handler, kafka_handler, save_searches) {
        info!("All tasks completed successfully");
        Ok(())
    } else {
        error!("One or more tasks failed");
        Err("One or more tasks failed".into())
    }
}

pub async fn download_list_data<S>(
    scraper: S,
    search_file_name: String,
    searches: Vec<HashMap<String, String>>,
) -> Result<(), String>
where
    S: ScraperTrait + ScrapeListTrait<MobileRecord> + Clone + Send + 'static,
{
    let (mut producer, mut receiver) = tokio::sync::mpsc::channel::<MobileRecord>(250);
    let (mut search_producer, search_receiver) =
        tokio::sync::mpsc::channel::<HashMap<String, String>>(250);

    let start_handler = tokio::spawn(async move {
        process_list(
            Box::new(scraper),
            searches,
            &mut producer,
            &mut search_producer,
        )
        .await
    });
    let send_to_kafka =
        tokio::spawn(async move { send_mobile_record_to_kafka(&mut receiver).await });

    let save_to_search_file =
        tokio::spawn(async move { log_search(search_receiver, search_file_name).await });
    if let (Ok(_), Ok(_), Ok(_)) = tokio::join!(start_handler, send_to_kafka, save_to_search_file) {
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

    use super::download_details;
    use crate::scraper::CarsBgScraper::CarsBGScraper;
    use crate::scraper::MobileBgScraper::MobileBGScraper;
    use crate::scraper::Traits::ScraperTrait;
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
        // download_details(
        //     scraper,
        //     "./resources/test-data/test_search.json".to_owned(),
        //     searches,
        // )
        // .await
        // .unwrap();
    }

    // #[tokio::test]
    // async fn test_run_mobilebg_scraper() {
    //     configure_log4rs(&LOG_CONFIG);
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

    //     let scraper = MobileBGScraper::new("https://www.mobile.bg/pcgi/mobile.cgi?", 250);
    //     let html = scraper.get_html(params.clone(), 1).await.unwrap();
    //     let slink = scraper.slink(&html).unwrap();
    //     params.clear();
    //     params.insert("act".to_owned(), "3".to_owned());
    //     params.insert("rub".to_string(), 1.to_string());
    //     params.insert("pubtype".to_string(), 1.to_string());
    //     params.insert("topmenu".to_string(), "1".to_string());
    //     params.insert("slink".to_owned(), slink.clone());
    //     let searches = vec![params];
    //     download_details(
    //         scraper,
    //         "./resources/test-data/test-search.json".to_owned(),
    //         searches,
    //     )
    //     .await
    //     .unwrap();
    // }

    #[tokio::test]
    async fn test_cars_bg_searches() {
        configure_log4rs(&LOG_CONFIG);
        let searches = cars_bg_new_searches();
        let mut total = 0;
        for search in searches {
            let html = CARS_BG_CRAWLER.get_html(search.clone(), 1).await.unwrap();
            let total_number = CARS_BG_CRAWLER.total_number(&html).unwrap();
            total += total_number;
            info!("total_number: {} for search: {:?}", total_number, search);
        }
        info!("total: {}", total);
    }

    #[tokio::test]
    async fn test_mobile_bg_searches() {
        configure_log4rs(&LOG_CONFIG);
        let searches = mobile_bg_new_searches();
        let slink_searches = super::to_slink_searches(searches).await;
        let mut total = 0;
        for search in slink_searches {
            let html = MOBILE_BG_CRAWLER.get_html(search.clone(), 1).await.unwrap();
            let total_number = MOBILE_BG_CRAWLER.total_number(&html).unwrap();
            total += total_number;
            info!("total_number: {} for search: {:?}", total_number, search);
        }
        info!("total: {}", total);
    }
}
