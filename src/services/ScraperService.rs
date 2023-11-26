use std::{collections::HashMap, time::Duration};

use log::{debug, error, info};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    time::{sleep, timeout},
};

use crate::{
    model::records::MobileRecord,
    scraper::ScraperTrait::{LinkId, ScraperTrait},
    services::SEMAPHORE,
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

// pub async fn start<T: ScraperTrait + Clone>(
//     scraper: Box<T>,
//     searches: Vec<HashMap<String, String>>,
//     link_producer: &mut Sender<LinkId>,
// ) -> Result<(), String>
// where
//     T: Send + 'static,
// {
//     let mut sum_total_number = 0;
//     let mut sum_sent_ids = 0;
//     for search in searches {
//         let total_number = scraper.total_number(search.clone()).await?;
//         // info!(
//         //     "Starting search: {:?}. Found {} vehicles",
//         //     search, total_number
//         // );
//         let cloned_scraper = scraper.clone();
//         let cloned_params = search.clone();
//         sum_total_number += total_number;
//         let number_of_pages = cloned_scraper.get_number_of_pages(total_number).unwrap();
//         for page_number in 1..number_of_pages {
//             let ids = cloned_scraper
//                 .get_listed_ids(cloned_params.clone(), page_number)
//                 .await
//                 .unwrap();
//             info!("*** Found ids: {}", ids.len());
//             for id in ids {
//                 if let Err(e) = link_producer.send(id).await {
//                     error!("Error sending id: {}", e);
//                 } else {
//                     sum_sent_ids += 1;
//                 }
//             }

//             if sum_sent_ids % 250 == 0 {
//                 info!("**** Sent ids: {}", sum_sent_ids);
//             }
//         }
//     }

//     info!("-------------------------------------------------");
//     info!("Total number of vehicles: {}", sum_total_number);
//     info!("Total number of sent LinkIds: {}", sum_sent_ids);
//     info!("-------------------------------------------------");

//     Ok(())
// }

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
        let total_number = scraper.total_number(search.clone()).await?;
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
                for id in ids {
                    if let Err(e) = cloned_producer.send(id).await {
                        error!("Error sending id: {}", e);
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
) -> Result<(), String>
where
    T: 'static,
{
    let mut counter = 0;
    let mut wait_counter = 0;
    loop {
        match timeout(Duration::from_millis(1000), link_receiver.recv()).await {
            Ok(Some(link)) => {
                let data = scraper.parse_details(link).await.unwrap();
                if !is_valid_data(&data) {
                    continue;
                }
                let record = MobileRecord::from(data);
                records_producer.send(record).await.unwrap();
                //sleep for 100 millis
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                debug!("Processed urls: {}", counter);
                counter += 1;
                if counter % 500 == 0 {
                    info!(">>> Processed urls: {}", counter);
                }
            }
            Ok(None) => {
                info!("No more links to process");
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

pub async fn save(
    mut records_producer: Receiver<MobileRecord>,
    file_name: String,
    source: &str,
) -> Result<(), String> {
    let mut counter = 0;
    let mut data = vec![];

    while let Some(mut record) = records_producer.recv().await {
        counter += 1;
        debug!("Processed data counter: {}", counter);
        record.id = format!("{}-{}", source, record.id);
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
