use std::{fmt::Debug, str::FromStr};

use log::{error, info};
use serde::Serialize;

use crate::{
    model::{
        AutoUncleVehicle::AutoUncleVehicle,
        Search::Search,
        VehicleDataModel::{BasicT, ChangeLogT, DetailsT, DownloadStatus, PriceT},
    },
    scraper::{
        AutouncleFRScraper::AutouncleFRScraper,
        AutouncleNLScraper::AutouncleNLScraper,
        AutouncleROScraper::AutouncleROScraper,
        CarsBgScraper::CarsBGScraper,
        MobileBgScraper::MobileBGScraper,
        Traits::{ScrapeListTrait, ScraperTrait},
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

pub async fn download_autouncle_data<S>(
    scraper: S,
    searches: Vec<Search>, // Same issue with U
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
    searches: Vec<Search>,
) -> Result<Vec<DownloadStatus>, String>
where
    S: ScraperTrait + ScrapeListTrait<T> + Clone + Send + 'static,
    T: BasicT + DetailsT + PriceT + ChangeLogT + Send + Serialize + Clone + Debug + 'static,
{
    let (mut producer, mut receiver) = tokio::sync::mpsc::channel::<T>(250);

    let start_handler =
        tokio::spawn(async move { process_list(Box::new(scraper), searches, &mut producer).await });
    let send_to_kafka = tokio::spawn(async move { send_data(&mut receiver).await });

    if let (Ok(scraped), Ok(_sent)) = tokio::join!(start_handler, send_to_kafka) {
        if let Ok(statuses) = scraped {
            Ok(statuses)
        } else {
            Err("Failed to scrape".into())
        }
    } else {
        error!("One or more tasks failed");
        Err("One or more tasks failed".into())
    }
}
