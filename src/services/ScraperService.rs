use std::{fmt::Debug, sync::Mutex, time::Duration};

use futures::future::join_all;
use log::{debug, error, info};
use serde::Serialize;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    time::{sleep, timeout},
};

use lazy_static::lazy_static;
use uuid::Uuid;

use crate::{
    BASE_INFO_CSV_FILE_NAME, BASE_INFO_PROTOBUF_FILE_NAME, DETAILS_CSV_NAME, DETAILS_PROTOBUF_NAME,
    PRICES_CSV_FILE_NAME, PRICES_PROTOBUF_FILE_NAME,
    kafka::{BASE_INFO_TOPIC, DETAILS_TOPIC, PRICE_TOPIC, broker},
    model::{
        Search::Search,
        VehicleDataModel::{
            BaseVehicleInfo, BasicT, ChangeLogT, DetailedVehicleInfo, DetailsT, DownloadStatus,
            Price, PriceT, ScrapedListData,
        },
        traits::{Identity, URLResource},
    },
    scraper::Traits::{RequestResponseTrait, ScrapeListTrait, ScraperTrait},
    writer::{
        flle_writer::file::FileWriter,
        kafka_writer::kafka::KafkaProducer,
        sink::{FormatterType, Sink, SinkType},
    },
};

lazy_static! {
    pub static ref TOTAL_COUNT: Mutex<u32> = Mutex::new(0);
}

#[derive(Debug, Clone)]
pub struct ScraperService<T: ScraperTrait + Clone> {
    pub scraper: T,
    pub file_name: String,
}

pub async fn process_list<S, T>(
    scraper: Box<S>,
    searches: Vec<Search>,
    producer: &mut Sender<T>,
) -> Result<Vec<DownloadStatus>, String>
where
    S: Send + ScraperTrait + ScrapeListTrait<T> + Clone + 'static,
    T: Send + BasicT + DetailsT + PriceT + Send + Clone + Serialize + Debug + 'static,
{
    let mut handlers = vec![];
    for search in searches {
        let html = scraper.get_html(search.clone(), 1).await?;
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

async fn download_all_found_results<S, T>(
    scraper: Box<S>,
    total_number: u32,
    search: Search,
    producer: Sender<T>,
) -> DownloadStatus
where
    S: Send + ScraperTrait + ScrapeListTrait<T> + Clone + 'static,
    T: Send + BasicT + DetailsT + PriceT + Clone + Serialize + Debug + 'static,
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
    for page_number in 1..=number_of_pages {
        let data = scraper
            .process_listed_results(search.clone(), page_number)
            .await;
        match data {
            Ok(ScrapedListData::Values(list)) => {
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
            Ok(ScrapedListData::SingleValue(value)) => {
                if (producer.send(value.clone()).await).is_err() {
                    error!("Error sending id: {:?}", value);
                }
            }
            _ => {
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

pub async fn process_details<S, Req, Res>(
    scraper: S,
    link_receiver: &mut Receiver<Req>,
    records_producer: &mut Sender<Res>,
) -> Result<(), String>
where
    S: Send + ScraperTrait + RequestResponseTrait<Req, Res> + Clone + 'static,
    Req: Send + Identity + Clone + Serialize + Debug + URLResource + 'static,
    Res: Send + Clone + Serialize + Debug + 'static,
{
    let mut counter = 0;
    let mut wait_counter = 0;
    let mut total_number = *TOTAL_COUNT.lock().unwrap();
    loop {
        match timeout(Duration::from_secs(1), link_receiver.recv()).await {
            Ok(Some(link)) => match scraper.handle_request(link.clone()).await {
                Ok(data) => {
                    wait_counter = 0;
                    if let Err(e) = records_producer.send(data).await {
                        error!("Error sending data: {}", e);
                    }
                    counter += 1;
                }
                Err(e) => {
                    error!("Error processing url: {}", e);
                    continue;
                }
            },
            Ok(None) => {
                break;
            }
            Err(_) => {
                wait_counter += 1;
                if wait_counter == 5 {
                    continue;
                }
            }
        }
        if total_number > 0 {
            let total_number = total_number as f32;
            let counter = counter as f32;
            let percent = counter * 100.0 / total_number;
            debug!(
                "Processing urls: {} / {}. Remaining: {}% ({})",
                counter,
                total_number,
                percent.round(),
                total_number - counter,
            );
        } else {
            total_number = *TOTAL_COUNT.lock().unwrap();
            debug!("Processing urls: {}", counter);
        }
    }

    Ok(())
}

pub async fn send_data<T: Sync + Send>(
    data_receiver: &mut Receiver<T>,
    sink_type: SinkType,
) -> Result<u32, String>
where
    T: Clone + BasicT + DetailsT + PriceT + Send + Sync + Debug + 'static,
{
    let mut counter = 0;
    let mut wait_counter = 0;

    let (base_sink, details_sink, price_sink): (
        Box<dyn Sink<BaseVehicleInfo>>,
        Box<dyn Sink<DetailedVehicleInfo>>,
        Box<dyn Sink<Price>>,
    ) = match sink_type {
        SinkType::Kafka => (
            Box::new(KafkaProducer::new(
                &broker(),
                BASE_INFO_TOPIC,
                FormatterType::Protobuf,
            )),
            Box::new(KafkaProducer::new(
                &broker(),
                DETAILS_TOPIC,
                FormatterType::Protobuf,
            )),
            Box::new(KafkaProducer::new(
                &broker(),
                PRICE_TOPIC,
                FormatterType::Protobuf,
            )),
        ),
        SinkType::ProtobufFile => (
            Box::new(FileWriter::new(
                &BASE_INFO_PROTOBUF_FILE_NAME,
                FormatterType::Protobuf,
            )),
            Box::new(FileWriter::new(
                &DETAILS_PROTOBUF_NAME,
                FormatterType::Protobuf,
            )),
            Box::new(FileWriter::new(
                &PRICES_PROTOBUF_FILE_NAME,
                FormatterType::Protobuf,
            )),
        ),
        SinkType::CsvFile => (
            Box::new(FileWriter::new(
                &BASE_INFO_CSV_FILE_NAME,
                FormatterType::Csv,
            )),
            Box::new(FileWriter::new(&DETAILS_CSV_NAME, FormatterType::Csv)),
            Box::new(FileWriter::new(&PRICES_CSV_FILE_NAME, FormatterType::Csv)),
        ),
    };

    loop {
        match timeout(Duration::from_secs(1), data_receiver.recv()).await {
            Ok(Some(data)) => {
                wait_counter = 0;
                let basic_info = BaseVehicleInfo::from(data.clone());
                let detais_info = DetailedVehicleInfo::from(data.clone());
                let price_info = Price::from(data.clone());
                info!("Sending data: {:?}", data.clone());
                base_sink
                    .write(basic_info)
                    .await
                    .map_err(|e| format!("Error sending base info: {}", e))?;
                details_sink
                    .write(detais_info)
                    .await
                    .map_err(|e| format!("Error sending details info: {}", e))?;
                price_sink
                    .write(price_info)
                    .await
                    .map_err(|e| format!("Error sending price info: {}", e))?;
                counter += 1;
                if (counter % 50) == 0 {
                    info!("Processed {} records", counter);
                    base_sink.flush().await?;
                    details_sink.flush().await?;
                    price_sink.flush().await?;
                }
            }

            Ok(None) => {
                info!("No more records to process. Total processed: {}", counter);
                base_sink.flush().await?;
                details_sink.flush().await?;
                price_sink.flush().await?;
                break;
            }

            Err(_e) => {
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

pub async fn process_list_and_send<S, Source>(
    scraper: Box<&S>,
    searches: Vec<Search>, // Same issue with U
    sender: &mut Sender<Source>,
) -> Result<(), String>
where
    S: Send + ScraperTrait + ScrapeListTrait<Source> + Clone + 'static,
    Source: BasicT + DetailsT + PriceT + ChangeLogT + Send + Clone + Serialize + Debug + 'static,
{
    let mut sum_total_number = 0;
    info!("Starting list processing. Searches: {}", searches.len());
    for search in searches {
        match process_search(scraper.clone(), search.clone(), sender.clone()).await {
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

async fn process_search<Scraper, Source>(
    scraper: Box<&Scraper>,
    search: Search, // Same issue with U
    sender: Sender<Source>,
) -> Result<u32, String>
where
    Scraper: Send + ScraperTrait + ScrapeListTrait<Source> + Clone + 'static,
    Source: Send + Clone + Serialize + Debug + 'static,
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
        match data {
            ScrapedListData::Values(list) => {
                for value in list {
                    if let Err(e) = sender.send(value.clone()).await {
                        error!("Error sending id: {}", e);
                    }
                }
            }
            ScrapedListData::SingleValue(value) => {
                if let Err(e) = sender.send(value.clone()).await {
                    error!("Error sending id: {}", e);
                }
            }
            ScrapedListData::Error(_) => {
                error!("Error getting data for page# : {}", 1);
            }
        }
        sleep(Duration::from_secs((page_number % 5) as u64)).await;
    }
    Ok(total_number)
}
