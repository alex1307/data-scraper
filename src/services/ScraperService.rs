use std::{collections::HashMap, fmt::Debug, sync::Mutex, time::Duration};

use futures::future::join_all;
use log::{debug, error, info};
use rand::Rng;
use serde::Serialize;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    time::{sleep, timeout},
};

use lazy_static::lazy_static;
use uuid::Uuid;

use crate::{
    kafka::{
        broker,
        KafkaProducer::{encode_message, send_message},
        BASE_INFO_TOPIC, CHANGE_LOG_TOPIC, DETAILS_TOPIC, PRICE_TOPIC,
    },
    model::{
        traits::{Identity, URLResource},
        AutoUncleVehicle,
        VehicleDataModel::{
            BaseVehicleInfo, BasicT, ChangeLogT, DetailedVehicleInfo, DetailsT, Price, PriceT,
            ScrapedListData, VehicleChangeLogInfo,
        },
        VehicleRecord::MobileRecord,
    },
    protos,
    scraper::Traits::{RequestResponseTrait, ScrapeListTrait, ScraperTrait},
    services::SearchBuilder::CRAWLER_KEY,
    writer::persistance::{MobileData, MobileDataWriter},
};

use super::ScraperAppService::DownloadStatus;

lazy_static! {
    pub static ref TOTAL_COUNT: Mutex<u32> = Mutex::new(0);
}

#[derive(Debug, Clone)]
pub struct ScraperService<T: ScraperTrait + Clone> {
    pub scraper: T,
    pub file_name: String,
}

pub async fn process_search<S, T>(
    scraper: Box<S>,
    search: HashMap<String, String>,
    producer: &mut Sender<T>,
) -> Result<tokio::task::JoinHandle<u32>, String>
where
    T: Send + BasicT + DetailsT + PriceT + ChangeLogT + Send + Clone + Serialize + Debug + 'static,
    S: Send + ScraperTrait + ScrapeListTrait<T> + Clone + 'static,
{
    let html = scraper.get_html(search.clone(), 1).await?;
    let total_number = scraper.total_number(&html)?;
    let cloned_scraper = scraper.clone();
    let cloned_params = search.clone();
    let cloned_producer = producer.clone();

    let handler = tokio::spawn(async move {
        let uuid = Uuid::new_v4().to_string();

        let number_of_pages = cloned_scraper.get_number_of_pages(total_number).unwrap();
        info!(
            "STARTING async session: {}. Expected number of results: {}. Number of pages: {}",
            uuid, total_number, number_of_pages
        );
        let mut actual_number = 0;
        for page_number in 1..number_of_pages {
            let data = cloned_scraper
                .process_listed_results(cloned_params.clone(), page_number)
                .await
                .unwrap();
            match data {
                ScrapedListData::Values(list) => {
                    let listing_wait_time: u64 = rand::thread_rng().gen_range(1_000..3_000);
                    sleep(Duration::from_millis(listing_wait_time as u64)).await;
                    actual_number += list.len() as u32;
                    for data in list {
                        if let Err(e) = cloned_producer.send(data.clone()).await {
                            error!("Error sending id: {}", e);
                        }
                    }
                }
                ScrapedListData::SingleValue(value) => {
                    if (cloned_producer.send(value.clone()).await).is_err() {
                        error!("Error sending id: {:?}", value);
                    }
                }
                ScrapedListData::Error(_) => {
                    error!("Error getting data for page# : {}", page_number);
                    continue;
                }
            }
        }
        info!(
            "FINISHED session: {}, processed vehicles: {}",
            uuid, actual_number
        );
        actual_number
    });

    Ok(handler)
}

pub async fn process_list<S, T>(
    scraper: Box<S>,
    searches: Vec<HashMap<String, String>>,
    producer: &mut Sender<T>,
) -> Result<Vec<DownloadStatus>, String>
where
    S: Send + ScraperTrait + ScrapeListTrait<T> + Clone + 'static,
    T: Send + BasicT + DetailsT + PriceT + ChangeLogT + Send + Clone + Serialize + Debug + 'static,
{
    let mut listed_vehicle_counter = 0;
    let mut handlers = vec![];
    for search in searches {
        let html = scraper.get_html(search.clone(), 1).await?;
        let total_number = scraper.total_number(&html)?;

        let cloned_scraper = scraper.clone();
        let cloned_params = search.clone();
        let cloned_producer = producer.clone();

        listed_vehicle_counter += total_number;
        let handler = tokio::spawn(async move {
            download_all_found_results(cloned_scraper, total_number, cloned_params, cloned_producer)
                .await
        });
        handlers.push(handler);
    }
    let results = join_all(handlers).await;
    let mut actual_vehicles = 0;
    let mut download_status = vec![];
    for result in results {
        let processed_vehicles = result.unwrap(); // Handle or log errors as needed
        actual_vehicles += processed_vehicles.actual;
        download_status.push(processed_vehicles);
    }
    TOTAL_COUNT
        .lock()
        .unwrap()
        .clone_from(&listed_vehicle_counter);
    info!("-------------------------------------------------");
    info!(
        "Total number of vehicles: {}. Processed: {}",
        TOTAL_COUNT.lock().unwrap(),
        actual_vehicles
    );
    info!("-------------------------------------------------");
    Ok(download_status)
}

async fn download_all_found_results<S, T>(
    scraper: Box<S>,
    total_number: u32,
    search: HashMap<String, String>,
    producer: Sender<T>,
) -> DownloadStatus
where
    S: Send + ScraperTrait + ScrapeListTrait<T> + Clone + 'static,
    T: Send + BasicT + DetailsT + PriceT + ChangeLogT + Send + Clone + Serialize + Debug + 'static,
{
    let uuid = Uuid::new_v4().to_string();

    let number_of_pages = scraper.get_number_of_pages(total_number).unwrap();
    let mut actual_number = 0;
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
    info!(
        "FINISHED session: {}, processed vehicles: {}",
        uuid, actual_number
    );
    let source = if let Some(source) = search.get(CRAWLER_KEY) {
        source
    } else {
        "n.a"
    };
    DownloadStatus {
        source: source.to_string(),
        search,
        listed: total_number,
        actual: actual_number,
    }
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
            Ok(Some(link)) => {
                match scraper.handle_request(link.clone()).await {
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
                }

                if counter % 500 == 0 {
                    info!(">>> Processed urls: {}", counter);
                }
            }
            Ok(None) => {
                info!("No more links to process. Total processed: {}", counter);
                break;
            }
            Err(e) => {
                wait_counter += 1;
                if wait_counter == 5 {
                    error!("Timeout receiving link: {}", e);
                    continue;
                }
            }
        }
        if total_number > 0 {
            let total_number = total_number as f32;
            let counter = counter as f32;
            let percent = counter * 100.0 / total_number;
            info!(
                "Processing urls: {} / {}. Remaining: {}% ({})",
                counter,
                total_number,
                percent.round(),
                total_number - counter,
            );
        } else {
            total_number = *TOTAL_COUNT.lock().unwrap();
            info!("Processing urls: {}", counter);
        }
    }

    Ok(())
}

pub async fn send_data<T: Clone + BasicT + DetailsT + PriceT + ChangeLogT>(
    data_receiver: &mut Receiver<T>,
) -> Result<u32, String> {
    let mut counter = 0;
    let mut wait_counter = 0;
    let broker = broker();
    let producer = crate::kafka::KafkaProducer::create_producer(&broker);

    loop {
        match timeout(Duration::from_secs(1), data_receiver.recv()).await {
            Ok(Some(data)) => {
                wait_counter = 0;
                let basic_info = BaseVehicleInfo::from(data.clone());
                let detais_info = DetailedVehicleInfo::from(data.clone());
                let price_info = Price::from(data.clone());
                let log_change_info = VehicleChangeLogInfo::from(data.clone());

                let basic_data = protos::vehicle_model::BaseVehicleInfo::from(basic_info);
                let details_data = protos::vehicle_model::DetailedVehicleInfo::from(detais_info);
                let price_data = protos::vehicle_model::Price::from(price_info);
                let log_change_data =
                    protos::vehicle_model::VehicleChangeLogInfo::from(log_change_info);

                let basic_encoded_message = encode_message(&basic_data).unwrap();
                let details_encoded_message = encode_message(&details_data).unwrap();
                let price_encoded_message = encode_message(&price_data).unwrap();
                let log_change_encoded_message = encode_message(&log_change_data).unwrap();

                send_message(&producer, BASE_INFO_TOPIC, basic_encoded_message).await;
                send_message(&producer, DETAILS_TOPIC, details_encoded_message).await;
                send_message(&producer, PRICE_TOPIC, price_encoded_message).await;
                send_message(&producer, CHANGE_LOG_TOPIC, log_change_encoded_message).await;

                counter += 1;
                if counter % 500 == 0 {
                    info!(">>> Processed urls: {}", counter);
                }
            }

            Ok(None) => {
                info!("No more records to process. Total processed: {}", counter);
                break;
            }
            Err(_e) => {
                wait_counter += 1;
                if wait_counter == 5 {
                    //error!("Timeout receiving link: {}", e);
                    continue;
                }
            }
        }
    }
    info!("All {} records are sent. Kafka producer is done.", counter);
    Ok(counter)
}

pub async fn send_mobilerecord_data(
    data_receiver: &mut Receiver<MobileRecord>,
) -> Result<(), String> {
    let mut counter = 0;
    let mut wait_counter = 0;
    let broker = broker();
    let producer = crate::kafka::KafkaProducer::create_producer(&broker);
    loop {
        match timeout(Duration::from_secs(1), data_receiver.recv()).await {
            Ok(Some(data)) => {
                wait_counter = 0;
                let basic_info = BaseVehicleInfo::from(data.clone());
                let detais_info = DetailedVehicleInfo::from(data.clone());
                let price_info = Price::from(data.clone());
                let log_change_info = VehicleChangeLogInfo::from(data.clone());

                let basic_data = protos::vehicle_model::BaseVehicleInfo::from(basic_info);
                let details_data = protos::vehicle_model::DetailedVehicleInfo::from(detais_info);
                let price_data = protos::vehicle_model::Price::from(price_info);
                let log_change_data =
                    protos::vehicle_model::VehicleChangeLogInfo::from(log_change_info);

                let basic_encoded_message = encode_message(&basic_data).unwrap();
                let details_encoded_message = encode_message(&details_data).unwrap();
                let price_encoded_message = encode_message(&price_data).unwrap();
                let log_change_encoded_message = encode_message(&log_change_data).unwrap();

                send_message(&producer, BASE_INFO_TOPIC, basic_encoded_message).await;
                send_message(&producer, DETAILS_TOPIC, details_encoded_message).await;
                send_message(&producer, PRICE_TOPIC, price_encoded_message).await;
                send_message(&producer, CHANGE_LOG_TOPIC, log_change_encoded_message).await;

                counter += 1;
                if counter % 500 == 0 {
                    info!(">>> Processed urls: {}", counter);
                }
            }

            Ok(None) => {
                info!("No more records to process. Total processed: {}", counter);
                break;
            }
            Err(e) => {
                wait_counter += 1;
                if wait_counter == 5 {
                    error!("Timeout receiving link: {}", e);
                    continue;
                }
                info!("Waiting for links to process");
            }
        }
    }

    Ok(())
}

pub async fn send_autonucle_kafka(
    data_receiver: &mut Receiver<AutoUncleVehicle::AutoUncleVehicle>,
) -> Result<(), String> {
    let mut counter = 0;
    let mut wait_counter = 0;
    let broker = broker();
    let producer = crate::kafka::KafkaProducer::create_producer(&broker);
    loop {
        match timeout(Duration::from_secs(1), data_receiver.recv()).await {
            Ok(Some(data)) => {
                wait_counter = 0;
                let basic_info = BaseVehicleInfo::from(data.clone());
                let detais_info = DetailedVehicleInfo::from(data.clone());
                let price_info = Price::from(data.clone());
                let log_change_info = VehicleChangeLogInfo::from(data.clone());

                let basic_data = protos::vehicle_model::BaseVehicleInfo::from(basic_info);
                let details_data = protos::vehicle_model::DetailedVehicleInfo::from(detais_info);
                let price_data = protos::vehicle_model::Price::from(price_info);
                let log_change_data =
                    protos::vehicle_model::VehicleChangeLogInfo::from(log_change_info);

                let basic_encoded_message = encode_message(&basic_data).unwrap();
                let details_encoded_message = encode_message(&details_data).unwrap();
                let price_encoded_message = encode_message(&price_data).unwrap();
                let log_change_encoded_message = encode_message(&log_change_data).unwrap();

                send_message(&producer, BASE_INFO_TOPIC, basic_encoded_message).await;
                send_message(&producer, DETAILS_TOPIC, details_encoded_message).await;
                send_message(&producer, PRICE_TOPIC, price_encoded_message).await;
                send_message(&producer, CHANGE_LOG_TOPIC, log_change_encoded_message).await;

                counter += 1;
                if counter % 500 == 0 {
                    info!(">>> Processed urls: {}", counter);
                }
            }

            Ok(None) => {
                info!("No more records to process. Total processed: {}", counter);
                break;
            }
            Err(e) => {
                wait_counter += 1;
                if wait_counter == 5 {
                    error!("Timeout receiving link: {}", e);
                    continue;
                }
            }
        }
    }

    Ok(())
}

pub async fn save<T: Clone + serde::Serialize>(
    mut receiver: Receiver<T>,
    file_name: String,
    threshold: u32,
) -> Result<(), String> {
    let mut counter = 0;
    let mut data = vec![];

    while let Some(record) = receiver.recv().await {
        counter += 1;
        debug!("Processed data counter: {}", counter);
        data.push(record.clone());
        if counter % threshold == 0 {
            save2file(&file_name, data.clone());
            data.clear();
        }
    }
    save2file(&file_name, data);
    Ok(())
}

pub fn save2file<T: Clone + serde::Serialize>(file_name: &str, data: Vec<T>) {
    info!(
        "Saving data number of records {} to file: {}",
        &data.len(),
        file_name
    );
    let new_data = MobileData::Payload(data);
    new_data.write_csv(file_name, false).unwrap();
}

pub async fn process_list_and_send<S, Source>(
    scraper: Box<&S>,
    searches: Vec<HashMap<String, String>>, // Same issue with U
    sender: &mut Sender<Source>,
) -> Result<(), String>
where
    S: Send + ScraperTrait + ScrapeListTrait<Source> + Clone + 'static,
    Source: BasicT + DetailsT + PriceT + ChangeLogT + Send + Clone + Serialize + Debug + 'static,
{
    let mut sum_total_number = 0;
    info!("Starting list processing. Searches: {}", searches.len());
    for search in searches {
        match process_search1(scraper.clone(), search.clone(), sender.clone()).await {
            Ok(total_number) => {
                sum_total_number += total_number;
            }
            Err(e) => {
                error!("Error processing search: {}", e);
                continue;
            }
        };
    }

    info!("-------------------------------------------------");
    info!("Total number of vehicles: {}", sum_total_number);
    info!("-------------------------------------------------");

    info!("All handlers finished");
    Ok(())
}

async fn process_search1<Scraper, Source>(
    scraper: Box<&Scraper>,
    search: HashMap<String, String>, // Same issue with U
    sender: Sender<Source>,
) -> Result<u32, String>
where
    Scraper: Send + ScraperTrait + ScrapeListTrait<Source> + Clone + 'static,
    Source: Send + Clone + Serialize + Debug + 'static,
{
    let html = scraper.get_html(search.clone(), 1).await?;
    let total_number = scraper.total_number(&html)?;
    info!(
        "Starting search: {:?}. Found {} vehicles",
        search, total_number
    );

    let cloned_params = search.clone();
    let number_of_pages = scraper.get_number_of_pages(total_number).unwrap();
    info!("number of pages: {}", number_of_pages);
    for page_number in 1..number_of_pages + 1 {
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
