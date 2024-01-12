use std::{collections::HashMap, fmt::Debug, str::FromStr, time::Duration};

use log::{error, info};
use serde::Serialize;
use tokio::time::timeout;

use crate::{
    model::{
        records::MobileRecord,
        traits::{Identity, URLResource},
        AutoUncleVehicle::AutoUncleVehicle,
        VehicleDataModel::{
            BaseVehicleInfo, DetailedVehicleInfo, LinkId, Price, VehicleChangeLogInfo,
        },
    },
    scraper::{
        AutouncleScraper::AutouncleScraper,
        CarGrScraper::CarGrScraper,
        CarsBgScraper::CarsBGScraper,
        MobileBgScraper::MobileBGScraper,
        Traits::{RequestResponseTrait, ScrapeListTrait, ScraperTrait},
    },
    services::ScraperService::{process_details, save},
    utils::helpers::create_empty_csv,
    AUTOUNCLE_ALL_SEARCHES_LOG, CARS_BG_ALL_FILE_NAME, CARS_BG_ALL_SEARCHES_LOG,
    CARS_BG_NEW_FILE_NAME, CARS_BG_NEW_SEARCHES_LOG, CONFIG, CREATED_ON, MOBILE_BG_ALL_FILE_NAME,
    MOBILE_BG_ALL_SEARCHES_LOG, MOBILE_BG_FILE_NAME, MOBILE_BG_NEW_SEARCHES_LOG,
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
    ScraperService::{log_search, process_list, process_list_and_send},
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
    match crawler {
        Crawlers::CarsBG(_) => {
            let searches = cars_bg_all_searches();
            download_list_data(
                CARS_BG_CRAWLER.clone(),
                CARS_BG_ALL_FILE_NAME.to_owned(),
                CARS_BG_ALL_SEARCHES_LOG.to_owned(),
                searches,
            )
            .await
        }
        Crawlers::MobileBG(_) => {
            let searches = mobile_bg_all_searches();
            let slink_searches = to_slink_searches(searches).await;
            download_list_data(
                MOBILE_BG_CRAWLER.clone(),
                MOBILE_BG_ALL_FILE_NAME.to_owned(),
                MOBILE_BG_ALL_SEARCHES_LOG.to_owned(),
                slink_searches,
            )
            .await
        }
        Crawlers::Autouncle(_) => {
            let searches: Vec<HashMap<String, String>> = autouncle_all_searches();
            download_autouncle_data(AUTOUNCLE_CRAWLER.clone(), searches).await
        }
    }
}

pub async fn download_new_vehicles(crawler: &str) -> Result<(), String> {
    let crawler = Crawlers::from_str(crawler)?;
    info!("crawler: {:?}", crawler);
    match crawler {
        Crawlers::CarsBG(_) => {
            let searches = cars_bg_new_searches();
            for s in searches.clone() {
                info!("search: {:?}", s);
            }
            download_details(
                CARS_BG_CRAWLER.clone(),
                CARS_BG_NEW_FILE_NAME.to_owned(),
                CARS_BG_NEW_SEARCHES_LOG.to_owned(),
                searches,
            )
            .await
        }
        Crawlers::MobileBG(_) => {
            let searches = mobile_bg_new_searches();
            let slink_searches = to_slink_searches(searches).await;
            download_details(
                MOBILE_BG_CRAWLER.clone(),
                MOBILE_BG_FILE_NAME.to_owned(),
                MOBILE_BG_NEW_SEARCHES_LOG.to_owned(),
                slink_searches,
            )
            .await
        }
        Crawlers::Autouncle(_) => todo!(),
    }
}

pub async fn download_details<S, REQ, RES>(
    scraper: S,
    file_name: String,
    file_search_name: String,
    searches: Vec<HashMap<String, String>>,
) -> Result<(), String>
where
    S: ScraperTrait
        + ScrapeListTrait<REQ>
        + RequestResponseTrait<REQ, RES>
        + Clone
        + Send
        + 'static,
    REQ: Send + Identity + Clone + Serialize + Debug + URLResource + 'static,
    RES: Send + Serialize + Clone + Debug + 'static,
{
    let (mut link_producer, mut link_receiver) = tokio::sync::mpsc::channel::<REQ>(250);
    let (mut record_producer, record_receiver) = tokio::sync::mpsc::channel::<RES>(250);
    let (mut search_producer, search_receiver) =
        tokio::sync::mpsc::channel::<HashMap<String, String>>(250);
    if create_empty_csv::<MobileRecord>(&file_name).is_err() {
        error!("Failed to create file {}", file_name.clone());
    }
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
    let save_to_file = tokio::spawn(async move { save(record_receiver, file_name, 100).await });
    let save_searches =
        tokio::spawn(async move { save(search_receiver, file_search_name, 1).await });

    if let (Ok(_), Ok(_), Ok(_), Ok(_)) =
        tokio::join!(start_handler, process_handler, save_to_file, save_searches)
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
    let (producer_base_info_producer, base_receiver) =
        tokio::sync::mpsc::channel::<BaseVehicleInfo>(250);
    let (producer_details_producer, details_receiver) =
        tokio::sync::mpsc::channel::<DetailedVehicleInfo>(250);
    let (producer_change_log_producer, change_log_receiver) =
        tokio::sync::mpsc::channel::<VehicleChangeLogInfo>(250);
    let (price_calculator_producer, price_receiver) = tokio::sync::mpsc::channel::<Price>(250);
    let (search_producer, search_receiver) =
        tokio::sync::mpsc::channel::<HashMap<String, String>>(250);

    let base_file = format!(
        "{}/vehicle-base-{}.csv",
        CONFIG.get_data_dir(),
        CREATED_ON.clone()
    );
    let details_file = format!(
        "{}/vehicle-details-{}.csv",
        CONFIG.get_data_dir(),
        CREATED_ON.clone()
    );
    let change_log_file = format!(
        "{}/vehicle-change-log{}.csv",
        CONFIG.get_data_dir(),
        CREATED_ON.clone()
    );

    let price_file = format!(
        "{}/vehicle-prices-{}.csv",
        CONFIG.get_data_dir(),
        CREATED_ON.clone()
    );
    if create_empty_csv::<VehicleChangeLogInfo>(&change_log_file).is_err() {
        error!("Failed to create file {}", change_log_file.clone());
    }

    if create_empty_csv::<BaseVehicleInfo>(&base_file).is_err() {
        error!("Failed to create file {}", base_file.clone());
    }

    if create_empty_csv::<DetailedVehicleInfo>(&details_file).is_err() {
        error!("Failed to create file {}", details_file.clone());
    }

    if create_empty_csv::<Price>(&price_file).is_err() {
        error!("Failed to create file {}", price_file.clone());
    }
    let start_handler = tokio::spawn(async move {
        process_list_and_send(
            Box::new(scraper),
            searches,
            &mut data_producer,
            search_producer,
        )
        .await
    });
    let mut counter = 0;
    let mut wait_counter = 0;
    let data_handler = tokio::spawn(async move {
        loop {
            counter += 1;
            info!("Processing urls: {}", counter);
            match timeout(Duration::from_secs(10), data_receiver.recv()).await {
                Ok(Some(data)) => {
                    let base = BaseVehicleInfo::from(data.clone());
                    let details = DetailedVehicleInfo::from(data.clone());
                    let change_log = VehicleChangeLogInfo::from(data.clone());
                    let price = Price::from(data.clone());
                    _ = producer_base_info_producer.send(base).await;
                    _ = producer_details_producer.send(details).await;
                    _ = producer_change_log_producer.send(change_log).await;
                    _ = price_calculator_producer.send(price).await;
                    info!("data: {:?}", data);
                }
                Ok(None) => {
                    info!("No more links to process. Total processed: {}", counter);
                    break;
                }
                Err(e) => {
                    wait_counter += 1;
                    if wait_counter == 3 {
                        error!("Timeout receiving link: {}", e);
                        continue;
                    } else {
                        info!("Waiting for links to process");
                    }
                }
            }
            if counter % 500 == 0 {
                info!(">>> Processed messages: {}", counter);
            }
        }
    });
    let save_to_base_file = tokio::spawn(async move { save(base_receiver, base_file, 100).await });
    let save_to_details_file =
        tokio::spawn(async move { save(details_receiver, details_file, 100).await });
    let save_log_change_file =
        tokio::spawn(async move { save(change_log_receiver, change_log_file, 100).await });
    let save_to_price_file =
        tokio::spawn(async move { save(price_receiver, price_file, 100).await });
    let save_searches = tokio::spawn(async move {
        log_search(search_receiver, AUTOUNCLE_ALL_SEARCHES_LOG.to_owned()).await
    });
    if let (Ok(_), Ok(_), Ok(_), Ok(_), Ok(_), Ok(_), Ok(_)) = tokio::join!(
        start_handler,
        data_handler,
        save_to_base_file,
        save_to_details_file,
        save_log_change_file,
        save_to_price_file,
        save_searches
    ) {
        info!("All tasks completed successfully");
        Ok(())
    } else {
        error!("One or more tasks failed");
        Err("One or more tasks failed".into())
    }
}

pub async fn download_list_data<S>(
    scraper: S,
    file_name: String,
    search_file_name: String,
    searches: Vec<HashMap<String, String>>,
) -> Result<(), String>
where
    S: ScraperTrait + ScrapeListTrait<LinkId> + Clone + Send + 'static,
{
    let (mut producer, receiver) = tokio::sync::mpsc::channel::<LinkId>(250);
    let (mut search_producer, search_receiver) =
        tokio::sync::mpsc::channel::<HashMap<String, String>>(250);
    if create_empty_csv::<MobileRecord>(&file_name).is_err() {
        error!("Failed to create file {}", file_name.clone());
    }

    if create_empty_csv::<MobileRecord>(&search_file_name).is_err() {
        error!("Failed to create file {}", search_file_name.clone());
    }
    let start_handler = tokio::spawn(async move {
        process_list(
            Box::new(scraper),
            searches,
            &mut producer,
            &mut search_producer,
        )
        .await
    });

    let save_to_file = tokio::spawn(async move { save(receiver, file_name, 100).await });
    let save_to_search_file =
        tokio::spawn(async move { save(search_receiver, search_file_name, 1).await });
    if let (Ok(_), Ok(_), Ok(_)) = tokio::join!(start_handler, save_to_file, save_to_search_file) {
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
        download_details(
            scraper,
            "./resources/test-data/test.csv".to_owned(),
            "./resources/test-data/test_search.json".to_owned(),
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
        let html = scraper.get_html(params.clone(), 1).await.unwrap();
        let slink = scraper.slink(&html).unwrap();
        params.clear();
        params.insert("act".to_owned(), "3".to_owned());
        params.insert("rub".to_string(), 1.to_string());
        params.insert("pubtype".to_string(), 1.to_string());
        params.insert("topmenu".to_string(), "1".to_string());
        params.insert("slink".to_owned(), slink.clone());
        let searches = vec![params];
        download_details(
            scraper,
            "./resources/test-data/test.csv".to_owned(),
            "./resources/test-data/test-search.json".to_owned(),
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
