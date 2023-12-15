use std::{collections::HashMap, fmt::Debug, time::Duration};

use log::{debug, error, info};
use rand::Rng;
use serde::Serialize;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    time::{sleep, timeout},
};

use crate::{
    model::{
        enums::MessageType,
        traits::{Identity, MessageTransform, URLResource},
        VehicleDataModel::ScrapedListData,
    },
    scraper::Traits::{RequestResponseTrait, ScrapeListTrait, ScraperTrait},
    writer::persistance::{MobileData, MobileDataWriter},
};

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
        let handler = tokio::spawn(async move {
            let number_of_pages = cloned_scraper.get_number_of_pages(total_number).unwrap();
            for page_number in 1..number_of_pages {
                let data = cloned_scraper
                    .get_listed_ids(cloned_params.clone())
                    .await
                    .unwrap();
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

    info!("-------------------------------------------------");
    info!("Total number of vehicles: {}", sum_total_number);
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
    loop {
        counter += 1;
        info!("Processing urls: {}", counter);
        match timeout(Duration::from_secs(300), link_receiver.recv()).await {
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
    }

    Ok(())
}



async fn process_search<Scraper, Source>(
    scraper: Box<Scraper>,
    search: HashMap<String, String>, // Same issue with U
    senders: Sender<MessageType>,
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
    for page_number in 1..number_of_pages {
        let data = cloned_scraper
            .get_listed_ids(cloned_params.clone())
            .await
            .unwrap();
        match data {
            ScrapedListData::Values(list) => {
                info!("*** Found ids: {}", &list.len());
                let listing_wait_time: u64 = rand::thread_rng().gen_range(3_000..10_000);
                sleep(Duration::from_millis(listing_wait_time as u64)).await;
                if let Err(e) = process_and_send(list.clone(), sender.clone()).await {
                        error!("Error sending id: {}", e);
                }
                
                
            }
            ScrapedListData::SingleValue(value) => {
                for sender in senders.clone() {
                    if let Err(e) = process_and_send(vec![value.clone()], sender.clone()).await {
                        error!("Error sending id: {}", e);
                    }
                }
            }
            ScrapedListData::Error(_) => {
                error!("Error getting data for page# : {}", page_number);
                continue;
            }
        }
    }
    Ok(total_number)
}


