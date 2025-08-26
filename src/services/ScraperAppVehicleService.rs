use std::{fmt::Debug, sync::Arc};

use log::{error, info};

use crate::{
    model::{
        Search::Search,
        VehicleDataModel::{DownloadStatus, Vehicle},
    },
    scraper::{BrowserController::BrowserController, VehicleTraits::VehicleScrapeTrait},
    writer::sink::SinkType,
};

use super::VehicleService::{process_list, send_data};

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
    sink_type: SinkType,
    browser: Option<Arc<BrowserController>>, // Same issue with U
) -> Result<(), String>
where
    S: VehicleScrapeTrait + Clone + Send + 'static,
{
    let (mut data_producer, mut data_receiver) = tokio::sync::mpsc::channel::<Vehicle>(1000);

    let start_handler = tokio::spawn(async move {
        process_list(Box::new(scraper), searches, &mut data_producer, browser).await
    });
    let kafka_handler = tokio::spawn(async move { send_data(&mut data_receiver, sink_type).await });

    if let (Ok(_), Ok(_)) = tokio::join!(start_handler, kafka_handler) {
        info!("All tasks completed successfully");
        Ok(())
    } else {
        error!("One or more tasks failed");
        Err("One or more tasks failed".into())
    }
}

pub async fn download_list_data<S>(
    scraper: S,
    searches: Vec<Search>,
    sink_type: SinkType,
    browser: Option<Arc<BrowserController>>, // Same issue with U
) -> Result<Vec<DownloadStatus>, String>
where
    S: VehicleScrapeTrait + Clone + Send + 'static,
{
    let (mut producer, mut receiver) = tokio::sync::mpsc::channel::<Vehicle>(250);

    let start_handler = tokio::spawn(async move {
        process_list(Box::new(scraper), searches, &mut producer, browser).await
    });

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
