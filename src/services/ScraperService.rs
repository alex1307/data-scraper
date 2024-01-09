use std::{collections::HashMap, fmt::Debug, sync::Mutex, time::Duration};

use log::{debug, error, info};
use rand::Rng;
use serde::Serialize;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    time::{sleep, timeout},
};

use lazy_static::lazy_static;

use crate::{
    model::{
        traits::{Identity, URLResource},
        VehicleDataModel::ScrapedListData,
    },
    scraper::Traits::{RequestResponseTrait, ScrapeListTrait, ScraperTrait},
    writer::persistance::{MobileData, MobileDataWriter},
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
    searches: Vec<HashMap<String, String>>,
    link_producer: &mut Sender<T>,
) -> Result<(), String>
where
    S: Send + ScraperTrait + ScrapeListTrait<T> + Clone + 'static,
    T: Send + Identity + URLResource + Clone + Serialize + Debug + 'static,
{
    let mut handlers = vec![];
    let mut sum_total_number = 0;
    for search in searches {
        let html = scraper.get_html(search.clone(), 1).await?;
        let total_number = scraper.total_number(&html)?.clone();
        info!(
            "Starting search: {:?}. Found {} vehicles",
            search, total_number
        );
        let cloned_scraper = scraper.clone();
        let cloned_params = search.clone();
        let cloned_producer = link_producer.clone();
        sum_total_number += total_number;
        info!("Total number of vehicles: {}", total_number);
        let handler = tokio::spawn(async move {
            let number_of_pages = cloned_scraper.get_number_of_pages(total_number).unwrap();
            info!("number of pages: {}", number_of_pages);
            for page_number in 1..number_of_pages {
                let data = cloned_scraper
                    .get_listed_ids(cloned_params.clone(), page_number)
                    .await
                    .unwrap();
                info!("*** Page number: {}", page_number);
                info!("*** Data: {:?}", data.clone());
                match data {
                    ScrapedListData::Values(list) => {
                        info!("*** Found ids: {}", list.len());
                        let listing_wait_time: u64 = rand::thread_rng().gen_range(3_000..10_000);
                        sleep(Duration::from_millis(listing_wait_time as u64)).await;
                        for id in list {
                            let advert_wait_time: u64 = rand::thread_rng().gen_range(3_000..7_000);
                            sleep(Duration::from_millis(advert_wait_time)).await;
                            if let Err(e) = cloned_producer.send(id.clone()).await {
                                error!("Error sending id: {}", e);
                            } else {
                                info!("Sent id: {}", &id.get_id());
                            }
                        }
                    }
                    ScrapedListData::SingleValue(value) => {
                        if let Err(_) = cloned_producer.send(value.clone()).await {
                            error!("Error sending id: {:?}", value);
                        }
                    }
                    ScrapedListData::Error(_) => {
                        error!("Error getting data for page# : {}", page_number);
                        continue;
                    }
                }
            }
        });
        handlers.push(handler);
    }
    TOTAL_COUNT.lock().unwrap().clone_from(&sum_total_number);
    info!("-------------------------------------------------");
    info!("Total number of vehicles: {}", TOTAL_COUNT.lock().unwrap());
    info!("-------------------------------------------------");

    for handler in handlers {
        info!("Waiting for handler to finish");
        handler.await.unwrap();
    }
    info!("All handlers finished");
    Ok(())
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
    let mut total_number = TOTAL_COUNT.lock().unwrap().clone();
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
                } else {
                    info!("Waiting for links to process");
                }
            }
        }
        if total_number > 0 {
            let total_number = total_number as f32;
            let counter = counter as f32;
            let percent = counter * 100.0/ total_number ;
            info!(
                "Processing urls: {} / {}. Remaining: {}% ({})",
                counter,
                total_number,
                percent.round(),
                total_number - counter,
            );
        } else {
            total_number = TOTAL_COUNT.lock().unwrap().clone();
            info!("Processing urls: {}", counter);
        }
    }

    Ok(())
}

pub async fn save<T: Clone + serde::Serialize>(
    mut receiver: Receiver<T>,
    file_name: String,
) -> Result<(), String> {
    let mut counter = 0;
    let mut data = vec![];

    while let Some(record) = receiver.recv().await {
        counter += 1;
        debug!("Processed data counter: {}", counter);
        data.push(record.clone());
        if counter % 50 == 0 {
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
    scraper: Box<S>,
    searches: Vec<HashMap<String, String>>, // Same issue with U
    sender: &mut Sender<Source>,            // Same issue with U
) -> Result<(), String>
where
    S: Send + ScraperTrait + ScrapeListTrait<Source> + Clone + 'static,
    Source: Send + Identity + Clone + Serialize + Debug + 'static,
{
    let mut sum_total_number = 0;
    info!("Starting list processing. Searches: {}", searches.len());
    for search in searches {
        match process_search(scraper.clone(), search, sender.clone()).await {
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

async fn process_search<Scraper, Source>(
    scraper: Box<Scraper>,
    search: HashMap<String, String>, // Same issue with U
    sender: Sender<Source>,
) -> Result<u32, String>
where
    Scraper: Send + ScraperTrait + ScrapeListTrait<Source> + Clone + 'static,
    Source: Send + Identity + Clone + Serialize + Debug + 'static,
{
    let html = scraper.get_html(search.clone(), 1).await?;
    let total_number = scraper.total_number(&html)?.clone();
    info!(
        "Starting search: {:?}. Found {} vehicles",
        search, total_number
    );
    let cloned_scraper = scraper.clone();
    let cloned_params = search.clone();
    let number_of_pages = cloned_scraper.get_number_of_pages(total_number).unwrap();
    info!("number of pages: {}", number_of_pages);
    for page_number in 1..number_of_pages + 1 {
        let data = cloned_scraper
            .get_listed_ids(cloned_params.clone(), page_number)
            .await
            .unwrap();
        match data {
            ScrapedListData::Values(list) => {
                info!("*** Found ids: {}", &list.len());
                for value in list {
                    if let Err(e) = sender.send(value.clone()).await {
                        error!("Error sending id: {}", e);
                    } else {
                        info!("Sent id: {}", &value.get_id());
                    }
                }
            }
            ScrapedListData::SingleValue(value) => {
                if let Err(e) = sender.send(value.clone()).await {
                    error!("Error sending id: {}", e);
                } else {
                    info!("Sent id: {}", &value.get_id());
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
