use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use futures::future::join_all;
use log::{debug, error, info};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    time::{sleep, timeout},
};

use lazy_static::lazy_static;
use uuid::Uuid;

#[cfg(feature = "kafka")]
use crate::kafka::{BASE_INFO_TOPIC, DETAILS_TOPIC, PRICE_TOPIC, VEHICLE_TOPIC, broker};
#[cfg(feature = "kafka")]
use crate::writer::kafka_writer::kafka::KafkaProducer;

use crate::{
    model::{
        Search::Search,
        VehicleDataModel::{DownloadStatus, Vehicle},
    },
    scraper::{BrowserController::BrowserController, VehicleTraits::VehicleScrapeTrait},
    utils::files::vehicle_file_name,
    writer::{
        flle_writer::file::FileWriter,
        sink::{FormatterType, Sink, SinkType},
    },
};

lazy_static! {
    pub static ref TOTAL_COUNT: Mutex<u32> = Mutex::new(0);
}

pub async fn process_list<S>(
    scraper: Box<S>,
    searches: Vec<Search>,
    producer: &mut Sender<Vehicle>,
    browser: Option<Arc<BrowserController>>,
) -> Result<Vec<DownloadStatus>, String>
where
    S: VehicleScrapeTrait + Clone + 'static,
{
    let mut handlers = vec![];
    for search in searches {
        let html = if let Some(browser) = browser.clone() {
            scraper.browse_html(browser, search.clone(), 1).await?
        } else {
            scraper.get_html(search.clone(), 1).await?
        };
        let total_number = scraper.total_number(&html)?;
        let cloned_scraper = scraper.clone();
        let cloned_params = search.clone();
        let cloned_producer = producer.clone();

        let handler = tokio::spawn(async move {
            download_all_found_results(cloned_scraper, total_number, cloned_params, cloned_producer)
                .await
        });
        handlers.push(handler);
    }
    let results = join_all(handlers).await;
    let mut download_status = vec![];
    for result in results {
        match result {
            Ok(status) => {
                info!("Download status: {:?}", status);
                TOTAL_COUNT.lock().unwrap().clone_from(&status.actual);
                download_status.push(status);
            }
            Err(e) => {
                error!("Error processing search: {}", e);
            }
        }
    }
    Ok(download_status)
}

pub async fn process_vehicle_list<T: VehicleScrapeTrait>(
    scraper: Box<T>,
    searches: Vec<Search>,
    producer: &mut Sender<Vehicle>,
) -> Result<Vec<DownloadStatus>, String> {
    let mut handlers = vec![];
    for search in searches {
        let html = scraper.get_html(search.clone(), 1).await?;
        let total_number = scraper.total_number(&html)?;
        let cloned_scraper = scraper.clone();
        let cloned_params = search.clone();
        let cloned_producer = producer.clone();

        let handler = tokio::spawn(async move {
            download_all_vehicles(cloned_scraper, total_number, cloned_params, cloned_producer)
                .await
        });
        handlers.push(handler);
    }
    let results = join_all(handlers).await;
    let mut download_status = vec![];
    for result in results {
        match result {
            Ok(status) => {
                info!("Download status: {:?}", status);
                TOTAL_COUNT.lock().unwrap().clone_from(&status.actual);
                download_status.push(status);
            }
            Err(e) => {
                error!("Error processing search: {}", e);
            }
        }
    }
    Ok(download_status)
}

async fn download_all_vehicles<T: VehicleScrapeTrait>(
    scraper: Box<T>,
    total_number: u32,
    search: Search,
    producer: Sender<Vehicle>,
) -> DownloadStatus {
    let uuid = Uuid::new_v4().to_string();

    let number_of_pages = scraper.get_number_of_pages(total_number).unwrap();
    let mut actual_number = 0;
    let url = search.url.clone();
    let hash = search.hash.clone();
    info!(
        "STARTING async session: {}. Expected number of results: {}. Number of pages: {}",
        uuid, total_number, number_of_pages
    );
    for page_number in 1..=number_of_pages {
        let data = scraper
            .process_listed_results(search.clone(), page_number)
            .await;
        match data {
            Ok(list) => {
                if list.len() < 25 {
                    info!("Get less data {} for page# : {}", list.len(), page_number);
                }
                actual_number += list.len() as u32;
                for data in list {
                    if let Err(e) = producer.send(data.clone()).await {
                        error!("Error sending id: {}", e);
                    }
                }
            }
            Err(_) => {
                error!("Error getting data for page# : {}", page_number);
                continue;
            }
        }
    }
    debug!(
        "FINISHED session: {}, processed vehicles: {}",
        uuid, actual_number
    );
    let source = search.source;
    let message = DownloadStatus {
        id: search.id,
        source: source.to_string(),
        url,
        listed: total_number,
        actual: actual_number,
        hash,
    };
    info!("Download status: {:?}", message);
    TOTAL_COUNT.lock().unwrap().clone_from(&actual_number);
    info!("TOTAL_COUNT: {}", *TOTAL_COUNT.lock().unwrap());
    message
}

async fn download_all_found_results<S>(
    scraper: Box<S>,
    total_number: u32,
    search: Search,
    producer: Sender<Vehicle>,
) -> DownloadStatus
where
    S: VehicleScrapeTrait + Clone + Send + 'static,
{
    let uuid = Uuid::new_v4().to_string();

    let number_of_pages = scraper.get_number_of_pages(total_number).unwrap();
    let mut actual_number = 0;
    let url = search.url.clone();
    let hash = search.hash.clone();
    info!(
        "STARTING async session: {}. Expected number of results: {}. Number of pages: {}",
        uuid, total_number, number_of_pages
    );
    sleep(Duration::from_secs(5)).await;
    for page_number in 1..=number_of_pages {
        let data = scraper
            .process_listed_results(search.clone(), page_number)
            .await;

        if data.is_err() {
            error!("Error getting data for page# : {}", page_number);
            continue;
        }
        let data = data.unwrap();
        if data.is_empty() {
            info!("Get empty data for page# : {}", page_number);
            continue;
        }
        if data.len() < 25 {
            info!("Get less data {} for page# : {}", data.len(), page_number);
        }
        actual_number += data.len() as u32;
        for data in data {
            if let Err(e) = producer.send(data.clone()).await {
                error!("Error sending id: {}", e);
            }
        }
    }

    debug!(
        "FINISHED session: {}, processed vehicles: {}",
        uuid, actual_number
    );
    let source = search.source;
    let message = DownloadStatus {
        id: search.id,
        source: source.to_string(),
        url,
        listed: total_number,
        actual: actual_number,
        hash,
    };
    info!("Download status: {:?}", message);
    TOTAL_COUNT.lock().unwrap().clone_from(&actual_number);
    info!("TOTAL_COUNT: {}", *TOTAL_COUNT.lock().unwrap());
    message
}

pub async fn send_data(
    data_receiver: &mut Receiver<Vehicle>,
    sink_type: SinkType,
) -> Result<u32, String> {
    let mut counter = 0;
    let mut wait_counter = 0;

    let vehicle_sink: Box<dyn Sink<Vehicle>> = match sink_type {
        #[cfg(feature = "kafka")]
        SinkType::Kafka => Box::new(KafkaProducer::new(
            &broker(),
            VEHICLE_TOPIC,
            FormatterType::Protobuf,
        )),
        #[cfg(not(feature = "kafka"))]
        SinkType::Kafka => panic!(
            "Kafka sink is not enabled. Please enable the 'kafka' feature in your Cargo.toml."
        ),
        SinkType::ProtobufFile => {
            let file_name = vehicle_file_name(&sink_type);
            Box::new(FileWriter::new(&file_name, FormatterType::Protobuf))
        }
        SinkType::CsvFile => {
            let file_name = vehicle_file_name(&sink_type);
            Box::new(FileWriter::new(&file_name, FormatterType::Csv))
        }
        #[cfg(feature = "postgres")]
        SinkType::PostgresDB => {
            info!("Using PostgresDB sink");
            info!("PostgresDB sink is enabled");
            Box::new(crate::writer::db_writer::db::DBWriter::new().await)
        }
        #[cfg(not(feature = "postgres"))]
        SinkType::PostgresDB => panic!(
            "Kafka sink is not enabled. Please enable the 'kafka' feature in your Cargo.toml."
        ),
    };

    loop {
        match timeout(Duration::from_secs(1), data_receiver.recv()).await {
            Ok(Some(data)) => {
                wait_counter = 0;
                vehicle_sink
                    .write(data.clone())
                    .await
                    .map_err(|e| format!("Error sending vehicle info: {}", e))?;
                counter += 1;
                if (counter % 50) == 0 {
                    info!("Processed {} records", counter);
                    vehicle_sink.flush().await?;
                }
            }

            Ok(None) => {
                info!("No more records to process. Total processed: {}", counter);
                vehicle_sink.flush().await?;
                break;
            }

            Err(_) => {
                wait_counter += 1;
                if wait_counter == 5 {
                    continue;
                }
            }
        }
    }

    info!("All {} records are sent. Sink completed.", counter);
    Ok(counter)
}

pub async fn process_list_and_send<S>(
    scraper: &S,
    searches: Vec<Search>, // Same issue with U
    sender: &mut Sender<Vehicle>,
) -> Result<(), String>
where
    S: Send + VehicleScrapeTrait + Clone + 'static,
{
    let mut sum_total_number = 0;
    info!("Starting list processing. Searches: {}", searches.len());
    for search in searches {
        match process_search(scraper, search.clone(), sender.clone()).await {
            Ok(total_number) => {
                sum_total_number += total_number;
            }
            Err(e) => {
                error!("Error processing search: {}", e);
                continue;
            }
        };
    }

    debug!("-------------------------------------------------");
    debug!("Total number of vehicles: {}", sum_total_number);
    debug!("-------------------------------------------------");

    info!("All handlers finished");
    Ok(())
}

async fn process_search<Scraper>(
    scraper: &Scraper,
    search: Search, // Same issue with U
    sender: Sender<Vehicle>,
) -> Result<u32, String>
where
    Scraper: Send + VehicleScrapeTrait + Clone + 'static,
{
    let html = scraper.get_html(search.clone(), 1).await?;
    let total_number = scraper.total_number(&html)?;
    debug!(
        "Starting search: {:?}. Found {} vehicles",
        search, total_number
    );

    let cloned_params = search.clone();
    let number_of_pages = scraper.get_number_of_pages(total_number).unwrap();
    debug!("number of pages: {}", number_of_pages);
    for page_number in 1..=number_of_pages {
        let data = scraper
            .process_listed_results(cloned_params.clone(), page_number)
            .await
            .unwrap();
        if data.is_empty() {
            info!("Get empty data for page# : {}", page_number);
            continue;
        }
        if data.len() < 25 {
            info!("Get less data {} for page# : {}", data.len(), page_number);
        }
        for data in data {
            if let Err(e) = sender.send(data.clone()).await {
                error!("Error sending id: {}", e);
            }
        }
        debug!("Processed page# : {}", page_number);
        if page_number % 5 == 0 {
            info!("Processed {} pages", page_number);
        }
        // Simulate a delay for the next page
        sleep(Duration::from_secs((page_number % 5) as u64)).await;
    }
    Ok(total_number)
}
