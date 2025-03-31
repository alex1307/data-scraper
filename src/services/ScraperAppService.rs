use std::fmt::Debug;

use log::{error, info};
use serde::Serialize;

use crate::{
    model::{
        AutouncleJsonModel::CarData,
        Search::Search,
        VehicleDataModel::{BasicT, DetailsT, DownloadStatus, PriceT},
    },
    scraper::Traits::{ScrapeListTrait, ScraperTrait},
    writer::sink::SinkType,
};

use super::ScraperService::{process_list, send_data};

#[derive(Debug, Clone)]
pub enum Crawlers {
    CarsBG(String),
    MobileBG(String),
    AutouncleRo(String),
    AutouncleNL(String),
    AutouncleFR(String),
}

pub async fn download_autouncle_data<S>(
    scraper: S,
    searches: Vec<Search>,
    sink_type: SinkType, // Same issue with U
) -> Result<(), String>
where
    S: ScraperTrait + ScrapeListTrait<CarData> + Clone + Send + 'static,
{
    let (mut data_producer, mut data_receiver) = tokio::sync::mpsc::channel::<CarData>(1000);

    let start_handler =
        tokio::spawn(
            async move { process_list(Box::new(scraper), searches, &mut data_producer).await },
        );
    let kafka_handler = tokio::spawn(async move { send_data(&mut data_receiver, sink_type).await });

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
    sink_type: SinkType,
) -> Result<Vec<DownloadStatus>, String>
where
    S: ScraperTrait + ScrapeListTrait<T> + Clone + Send + 'static,
    T: BasicT + DetailsT + PriceT + Send + Sync + Serialize + Clone + Debug + 'static,
{
    let (mut producer, mut receiver) = tokio::sync::mpsc::channel::<T>(250);

    let start_handler =
        tokio::spawn(async move { process_list(Box::new(scraper), searches, &mut producer).await });

    let sink_handler = tokio::spawn(async move { send_data(&mut receiver, sink_type).await });

    if let (Ok(scraped), Ok(_sent)) = tokio::join!(start_handler, sink_handler) {
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
