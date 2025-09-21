use std::{fmt::Debug, sync::Arc, time::Instant};

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

/// Summary of a finished scraping job
#[derive(Debug, Clone)]
pub struct JobStatus {
    pub source: String,           // e.g. "autouncle.ro" / "mobile.bg"
    pub cfg_path: Option<String>, // YAML path or folder
    pub url: Option<String>,      // optional main search URL for quick inspection
    pub listed: u32,              // total items listed by pages
    pub actual: u32,              // total items actually processed/persisted
    pub duration_s: f64,          // duration in seconds
}

/// Error details when a job fails early or downstream returns Err
#[derive(Debug, Clone)]
pub struct JobError {
    pub source: String,
    pub cfg_path: Option<String>,
    pub url: Option<String>,
    pub message: String, // human-readable error message
}

pub type JobResult = Result<JobStatus, JobError>;

/// High-level orchestrator that runs a scraping job and returns a typed result
/// instead of an opaque tuple. Keeps the orchestration close to the list-download logic.
pub async fn run_job<S>(
    scraper: S,
    searches: Vec<Search>,
    sink_type: SinkType,
    browser: Option<Arc<BrowserController>>,
    source: String,
    cfg_path: Option<String>,
    search_url: Option<String>,
) -> JobResult
where
    S: VehicleScrapeTrait + Clone + Send + 'static,
{
    let started = Instant::now();
    match download_list_data(scraper, searches, sink_type, browser).await {
        Ok(statuses) => {
            let mut listed: u32 = 0;
            let mut actual: u32 = 0;
            for s in statuses {
                listed = listed.saturating_add(s.listed);
                actual = actual.saturating_add(s.actual);
            }
            Ok(JobStatus {
                source,
                cfg_path,
                url: search_url,
                listed,
                actual,
                duration_s: (started.elapsed().as_millis() as f64) / 1000.0,
            })
        }
        Err(e) => Err(JobError {
            source,
            cfg_path,
            url: search_url,
            message: e,
        }),
    }
}
