use std::{collections::HashMap, fmt::Debug, str::FromStr};

use log::{error, info};
use serde::Serialize;

use crate::{
    model::{
        AutoUncleVehicle::AutoUncleVehicle,
        VehicleDataModel::{BasicT, ChangeLogT, DetailsT, PriceT},
    },
    scraper::{
        AutouncleFRScraper::AutouncleFRScraper,
        AutouncleNLScraper::AutouncleNLScraper,
        AutouncleROScraper::AutouncleROScraper,
        CarsBgScraper::CarsBGScraper,
        MobileBgScraper::MobileBGScraper,
        Traits::{ScrapeListTrait, ScraperTrait},
    },
    services::SearchBuilder::{
        build_autouncle_fr_searches, build_autouncle_nl_searches, build_autouncle_ro_searches,
        build_cars_bg_all_searches, build_mobile_bg_all_searches,
    },
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref MOBILE_BG_CRAWLER: MobileBGScraper =
        MobileBGScraper::new("https://www.mobile.bg/obiavi/avtomobili-dzhipove", 250);
    pub static ref CARS_BG_CRAWLER: CarsBGScraper = CarsBGScraper::new("https://www.cars.bg", 250);
    pub static ref AUTOUNCLE_RO_CRAWLER: AutouncleROScraper =
        AutouncleROScraper::new("https://www.autouncle.ro/en/cars_search?", 250);
    pub static ref AUTOUNCLE_NL_CRAWLER: AutouncleNLScraper =
        AutouncleNLScraper::new("https://www.autouncle.nl/en/cars_search?", 250);
    pub static ref AUTOUNCLE_FR_CRAWLER: AutouncleFRScraper =
        AutouncleFRScraper::new("https://www.autouncle.fr/en/cars_search?", 250);
}

use super::ScraperService::{process_list, send_data};
#[derive(Debug, Clone)]

pub struct DownloadStatus {
    pub source: String,
    pub search: HashMap<String, String>,
    pub listed: u32,
    pub actual: u32,
}
#[derive(Debug, Clone)]
pub enum Crawlers {
    CarsBG(String),
    MobileBG(String),
    AutouncleRo(String),
    AutouncleNL(String),
    AutouncleFR(String),
}

impl FromStr for Crawlers {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "cars.bg" => Ok(Crawlers::CarsBG(r#"https://www.cars.bg"#.to_owned())),
            "cars_bg" => Ok(Crawlers::CarsBG(r#"https://www.cars.bg"#.to_owned())),
            "cars" => Ok(Crawlers::CarsBG(r#"https://www.cars.bg"#.to_owned())),
            "mobile.bg" => Ok(Crawlers::MobileBG(
                r#"https://www.mobile.bg/pcgi/mobile.cgi?"#.to_owned(),
            )),
            "mobile_bg" => Ok(Crawlers::MobileBG(
                r#"https://www.mobile.bg/pcgi/mobile.cgi?"#.to_owned(),
            )),
            "mobile" => Ok(Crawlers::MobileBG(
                r#"https://www.mobile.bg/pcgi/mobile.cgi?"#.to_owned(),
            )),
            "autouncle" => Ok(Crawlers::AutouncleRo(
                r#"https://www.autouncle.ro"#.to_owned(),
            )),
            "autouncle.ro" => Ok(Crawlers::AutouncleRo(
                r#"https://www.autouncle.ro"#.to_owned(),
            )),
            _ => Err("Invalid crawler".into()),
        }
    }
}

pub async fn download_all(crawler: &str) -> Result<(), String> {
    let crawler = Crawlers::from_str(crawler)?;
    //info!("Starting crawler (all): {:?}", crawler);
    match crawler {
        Crawlers::CarsBG(_) => {
            info!("Starting cars.bg");
            let searches = build_cars_bg_all_searches();
            let chunks = searches.chunks(10);
            info!("Starting list processing. chunks: {}", chunks.len());
            let mut max_10_searches = vec![];
            for c in chunks {
                max_10_searches.clear();
                for search in c {
                    max_10_searches.push(search.clone());
                }
                let _ = download_list_data(CARS_BG_CRAWLER.clone(), max_10_searches.clone()).await;
            }
            Ok(())
        }
        Crawlers::MobileBG(_) => {
            info!("Starting mobile.bg");
            let searches = build_mobile_bg_all_searches();
            let chunks = searches.chunks(10);
            info!("Starting list processing. chunks: {}", chunks.len());
            let mut max_10_searches = vec![];
            for c in chunks {
                max_10_searches.clear();
                for search in c {
                    max_10_searches.push(search.clone());
                }
                let _ =
                    download_list_data(MOBILE_BG_CRAWLER.clone(), max_10_searches.clone()).await;
            }
            Ok(())
        }
        Crawlers::AutouncleRo(_) => {
            info!("Starting autouncle.ro");
            let searches: Vec<HashMap<String, String>> = build_autouncle_ro_searches();
            let splitted_searches = searches.chunks(10);
            let mut max_10_searches = vec![];
            for chunks in splitted_searches {
                max_10_searches.clear();
                for search in chunks {
                    max_10_searches.push(search.clone());
                }
                let _ =
                    download_list_data(AUTOUNCLE_RO_CRAWLER.clone(), max_10_searches.clone()).await;
            }
            Ok(())
        }
        Crawlers::AutouncleNL(_) => {
            info!("Starting autouncle.nl");
            let searches: Vec<HashMap<String, String>> = build_autouncle_nl_searches();
            let splitted_searches = searches.chunks(10);
            let mut max_10_searches = vec![];
            for chunks in splitted_searches {
                max_10_searches.clear();
                for search in chunks {
                    max_10_searches.push(search.clone());
                }
                let _ =
                    download_list_data(AUTOUNCLE_NL_CRAWLER.clone(), max_10_searches.clone()).await;
            }
            Ok(())
        }
        Crawlers::AutouncleFR(_) => {
            info!("Starting autouncle.fr");
            let searches: Vec<HashMap<String, String>> = build_autouncle_fr_searches();
            let splitted_searches = searches.chunks(10);
            let mut max_10_searches = vec![];
            for chunks in splitted_searches {
                max_10_searches.clear();
                for search in chunks {
                    max_10_searches.push(search.clone());
                }
                let _ =
                    download_list_data(AUTOUNCLE_FR_CRAWLER.clone(), max_10_searches.clone()).await;
            }
            Ok(())
        }
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

    let start_handler =
        tokio::spawn(
            async move { process_list(Box::new(scraper), searches, &mut data_producer).await },
        );
    let kafka_handler = tokio::spawn(async move { send_data(&mut data_receiver).await });

    if let (Ok(_), Ok(_)) = tokio::join!(start_handler, kafka_handler) {
        info!("All tasks completed successfully");
        Ok(())
    } else {
        error!("One or more tasks failed");
        Err("One or more tasks failed".into())
    }
}

pub async fn download_list_data<S, T>(
    scraper: S,
    searches: Vec<HashMap<String, String>>,
) -> Result<(), String>
where
    S: ScraperTrait + ScrapeListTrait<T> + Clone + Send + 'static,
    T: BasicT + DetailsT + PriceT + ChangeLogT + Send + Serialize + Clone + Debug + 'static,
{
    let (mut producer, mut receiver) = tokio::sync::mpsc::channel::<T>(250);

    let start_handler =
        tokio::spawn(async move { process_list(Box::new(scraper), searches, &mut producer).await });
    let send_to_kafka = tokio::spawn(async move { send_data(&mut receiver).await });

    if let (Ok(scraped), Ok(sent)) = tokio::join!(start_handler, send_to_kafka) {
        if let Ok(statuses) = scraped {
            let mut total_listed = 0;
            let mut total_actual = 0;
            let source = statuses[0].source.clone();
            for status in statuses {
                info!("-> {:?}", status);
                total_listed += status.listed;
                total_actual += status.actual;
            }
            if let Ok(counter) = sent {
                info!(
                    "Overall report for {}: listed {}, actual: {}, sent: {}",
                    source, total_listed, total_actual, counter
                );
            }
        }

        Ok(())
    } else {
        error!("One or more tasks failed");
        Err("One or more tasks failed".into())
    }
}

#[cfg(test)]
mod app_test {
    use log::info;

    use crate::scraper::Traits::ScraperTrait;
    use crate::services::ScraperAppService::CARS_BG_CRAWLER;
    use crate::services::Searches::cars_bg_new_searches;
    use crate::utils::helpers::configure_log4rs;
    use crate::LOG_CONFIG;

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
}
