use std::{collections::HashMap, time::Duration};

use log::{debug, error, info};
use rand::Rng;
use reqwest::header;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    time::{sleep, timeout},
};

use crate::{
    model::records::MobileRecord,
    scraper::ScraperTrait::{LinkId, ScraperTrait},
    writer::persistance::{MobileData, MobileDataWriter},
};

#[derive(Debug, Clone)]
pub struct ScraperService<T: ScraperTrait + Clone> {
    pub scraper: T,
    pub file_name: String,
}

fn is_valid_data(data: &HashMap<String, String>) -> bool {
    ["id", "make", "engine", "gearbox"]
        .iter()
        .all(|&key| data.contains_key(key))
}

pub async fn start<T: ScraperTrait + Clone>(
    scraper: Box<T>,
    searches: Vec<HashMap<String, String>>,
    link_producer: &mut Sender<LinkId>,
) -> Result<(), String>
where
    T: Send + 'static,
{
    let mut handlers = vec![];
    let mut sum_total_number = 0;
    for search in searches {
        let url = scraper.search_url(None, search.clone(), 0);
        let total_number = scraper.total_number(search.clone(), c1).await?.clone();
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
                let ids = cloned_scraper
                    .get_listed_ids(cloned_params.clone(), page_number)
                    .await
                    .unwrap();
                info!("*** Found ids: {}", ids.len());
                info!("*** Found ids: {:?}", ids);
                let listing_wait_time: u64 = rand::thread_rng().gen_range(3_000..10_000);
                sleep(Duration::from_millis(listing_wait_time as u64)).await;
                for id in ids {
                    let advert_wait_time: u64 = rand::thread_rng().gen_range(3_000..7_000);
                    sleep(Duration::from_millis(advert_wait_time)).await;
                    if let Err(e) = cloned_producer.send(id.clone()).await {
                        error!("Error sending id: {}", e);
                    } else {
                        info!("Sent id: {}", &id.id);
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

pub async fn process<T: ScraperTrait + Clone + Send>(
    scraper: T,
    link_receiver: &mut Receiver<LinkId>,
    records_producer: &mut Sender<MobileRecord>,
    headers: HashMap<String, String>,
) -> Result<(), String>
where
    T: 'static,
{
    let mut counter = 0;
    let mut wait_counter = 0;
    loop {
        counter += 1;
        info!("Processing urls: {}", counter);
        match timeout(Duration::from_secs(300), link_receiver.recv()).await {
            Ok(Some(link)) => {
                let data = scraper.parse_details(link, headers.clone()).await.unwrap();
                if !is_valid_data(&data) {
                    continue;
                }
                let record = MobileRecord::from(data);
                records_producer.send(record).await.unwrap();
                //sleep for 100 millis
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                debug!("Processed urls: {}", counter);

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
                    break;
                } else {
                    info!("Waiting for links to process");
                }
            }
        }
    }

    info!(">>> Processed urls: {}", counter);
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

fn save2file<T: Clone + serde::Serialize>(file_name: &str, data: Vec<T>) {
    info!(
        "Saving data number of records {} to file: {}",
        &data.len(),
        file_name
    );
    let new_data = MobileData::Payload(data);
    new_data.write_csv(file_name, false).unwrap();
}
