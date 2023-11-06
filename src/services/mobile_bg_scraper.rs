use std::{collections::HashSet, error::Error, fmt::Debug, vec};

use crossbeam_channel::{Receiver, Sender};
use futures::StreamExt;
use log::{debug, error, info};
use reqwest::Url;
use tokio::{sync::Mutex, task::JoinHandle};

use crate::{
    model::{
        enums::SaleType,
        id_list::IDList,
        records::MobileRecord,
        search_metadata::{asearch, asearches, SearchMetadata},
    },
    scraper::mobile_bg::{details2map, get_links},
    utils::helpers::{create_empty_csv, crossbeam_utils::to_stream, mobile_search_url},
    writer::persistance::{MobileData, MobileDataWriter},
    ARCHIVE_FILE_NAME, DELETED_FILE_NAME, DETAILS_URL, FOR_UPDATE_FILE_NAME, INSALE_FILE_NAME,
    LISTING_URL, METADATA_FILE_NAME, UPDATED_FILE_NAME, UPDATED_VEHICLES_FILE_NAME,
};
use lazy_static::lazy_static;

use super::file_processor::{self, DataProcessor};
pub const FLUSH_SIZE: usize = 400;
lazy_static! {
    static ref LISTING_MUTEX: Mutex<()> = Mutex::new(());
    static ref DETAILS_MUTEX: Mutex<()> = Mutex::new(());
}

pub async fn update() -> Result<(), Box<dyn Error>> {
    if create_empty_csv::<MobileRecord>(&UPDATED_VEHICLES_FILE_NAME).is_err() {
        error!(
            "Failed to create file {}",
            UPDATED_VEHICLES_FILE_NAME.clone()
        );
    }

    if create_empty_csv::<IDList>(&UPDATED_FILE_NAME).is_err() {
        error!("Failed to create file {:?}", UPDATED_FILE_NAME.clone());
    }

    if create_empty_csv::<IDList>(&DELETED_FILE_NAME).is_err() {
        error!("Failed to create file {:?}", DELETED_FILE_NAME.clone());
    }

    let update_processor = DataProcessor::<IDList>::from_files(vec![&FOR_UPDATE_FILE_NAME]);
    let update_data = update_processor.get_ids().clone();
    let cloned_ids = update_data.clone();
    let search_all = asearch(SaleType::INSALE, 1, 9_999_999).await;
    let mut urls = HashSet::new();

    let (link_producer, mut link_consumer) = crossbeam::channel::unbounded::<String>();
    let (record_producer, mut record_consumer) = crossbeam::channel::unbounded::<MobileRecord>();

    let producer_task = tokio::spawn(async move {
        for id in update_data {
            let url = format!(
                "{}&adv={}&slink={}",
                DETAILS_URL,
                id,
                search_all.slink.clone()
            );
            link_producer.send(url.clone()).unwrap();
            urls.insert(url);
        }
        info!("Total number of urls: {}", urls.len());
    });

    let process_links_task = tokio::spawn(async move {
        process_links(&mut link_consumer, record_producer).await;
    });

    let save_to_file_task = tokio::spawn(async move {
        save(&UPDATED_VEHICLES_FILE_NAME, &mut record_consumer).await;
    });

    if let (Ok(_), Ok(_), Ok(_)) =
        tokio::join!(producer_task, process_links_task, save_to_file_task)
    {
        info!("All tasks completed successfully");
    } else {
        error!("One or more tasks failed");
        return Err("One or more tasks failed".into());
    }

    let updated_processor =
        DataProcessor::<MobileRecord>::from_files(vec![&UPDATED_VEHICLES_FILE_NAME]);
    let updated_data = updated_processor.get_ids().clone();
    let deleted_ids = cloned_ids
        .difference(&updated_data)
        .cloned()
        .collect::<Vec<String>>();
    save2file(&DELETED_FILE_NAME, deleted_ids);
    Ok(())
}

pub async fn scrape() -> Result<(), Box<dyn Error>> {
    if create_empty_csv::<MobileRecord>(&INSALE_FILE_NAME).is_err() {
        error!("Failed to create file {}", INSALE_FILE_NAME.clone());
    }

    if create_empty_csv::<MobileRecord>(&ARCHIVE_FILE_NAME).is_err() {
        error!("Failed to create file {:?}", ARCHIVE_FILE_NAME.clone());
    }

    if create_empty_csv::<SearchMetadata>(&METADATA_FILE_NAME).is_err() {
        error!("Failed to create file {:?}", METADATA_FILE_NAME.clone());
    }

    let (link_producer, mut link_consumer) = crossbeam::channel::unbounded::<String>();
    let (filter_producer, mut filter_consumer) = crossbeam::channel::unbounded::<String>();
    let (details_producer, mut details_consumer) = crossbeam::channel::unbounded::<MobileRecord>();
    let start = tokio::spawn(async move {
        start_searches(link_producer).await;
    });
    let filter_task = tokio::spawn(async move {
        filter_links(&mut link_consumer, filter_producer).await;
    });
    let scrape_task = tokio::spawn(async move {
        process_links(&mut filter_consumer, details_producer).await;
    });
    let save_task = tokio::spawn(async move {
        save(&INSALE_FILE_NAME, &mut details_consumer).await;
    });

    if let (Ok(_), Ok(_), Ok(_), Ok(_)) = tokio::join!(save_task, scrape_task, filter_task, start) {
        info!("All tasks completed successfully");
        Ok(())
    } else {
        error!("One or more tasks failed");
        Err("One or more tasks failed".into())
    }
}

pub async fn spawn_sequentially(url: String, sender: Sender<String>) -> JoinHandle<()> {
    let _guard = LISTING_MUTEX.lock().await;
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    info!("spawn_sequentially");
    tokio::spawn(async move {
        links(&url, sender).await;
    })
}

async fn filter_links(consumer: &mut Receiver<String>, producer: Sender<String>) {
    let stream = Box::pin(to_stream(consumer));
    futures::pin_mut!(stream);
    let mut counter = 0;
    let mut ids = HashSet::new();
    info!("Records loaded today: {}", ids.len());
    while let Some(url) = stream.next().await {
        if let Ok(url) = Url::parse(&url) {
            if let Some(adv_value) = url.query_pairs().find(|(key, _)| key == "adv") {
                if ids.contains(&adv_value.1.to_string()) {
                    continue;
                }
                ids.insert(adv_value.1.to_string());
                producer.send(url.to_string()).unwrap();
                counter += 1;
            }
        } else {
            error!("Failed to parse url: {}", url);
        }
    }
    info!("Processed urls: {}", counter);
}

async fn links(url: &str, sender: Sender<String>) {
    let links = get_links(url).await;
    for link in links {
        sender.send(link).unwrap();
    }
}

async fn start_searches(link_producer: Sender<String>) {
    let mut all = vec![];
    let searches = asearches().await;
    info!("searches: {:?}", searches.len());
    all.extend(searches.clone());
    for meta in all.iter() {
        info!("{:?}", meta.clone());
    }
    let mut meta_data_processor: DataProcessor<SearchMetadata> =
        file_processor::DataProcessor::from_files(vec![&METADATA_FILE_NAME]);
    meta_data_processor.process(&all, None);
    let mut tasks = Vec::new();
    let mut counter = 0;
    for search in searches.iter() {
        counter += search.total_number;
        let pages: Vec<String> = (1..=search.page_numbers()).map(|n| n.to_string()).collect();
        //let pages = vec![1.to_string()];
        for page in pages {
            let url = mobile_search_url(
                LISTING_URL,
                &page,
                &search.slink,
                crate::model::enums::SaleType::NONE,
                0,
                0,
            );
            let task = spawn_sequentially(url, link_producer.clone());
            tasks.push(task);
        }
    }
    info!("Total number of links: {}", counter);

    for task in tasks {
        task.await;
    }
}

async fn process_links(input: &mut Receiver<String>, output: Sender<MobileRecord>) {
    let stream = Box::pin(to_stream(input));
    futures::pin_mut!(stream);
    let mut counter = 0;
    let mut urls = HashSet::new();
    while let Some(url) = stream.next().await {
        debug!("url: {}", url.clone());
        let mut not_found = 0;
        let data = details2map(url.as_str()).await;
        if data.is_empty()
            || !data.contains_key("id")
            || !data.contains_key("make")
            || !data.contains_key("engine")
            || !data.contains_key("gearbox")
        {
            urls.insert(url);
            if urls.len() % 100 == 0 {
                for u in &urls{
                    info!("{}", u);
                }
                urls.clear();
            }
            not_found += 1;
            info!("Total not found urls: {}", not_found);
            continue;
        }
        let record = MobileRecord::from(data);
        output.send(record).unwrap();
        //sleep for 100 millis
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        info!("Processed urls: {}", counter);
        counter += 1;
    }
    info!("Processed urls: {}", counter);
}

fn save2file<T: Clone + Debug + serde::Serialize>(file_name: &str, data: Vec<T>) {
    let new_data = MobileData::Payload(data);
    info!("Saving data to file: {}", file_name);
    new_data.write_csv(file_name, false).unwrap();
}

pub async fn save_active_adverts(file_name: &str, input: &mut Receiver<MobileRecord>) {
    let stream = Box::pin(to_stream(input));
    futures::pin_mut!(stream);
    let mut counter = 0;
    let mut new_values = vec![];
    while let Some(data) = stream.next().await {
        new_values.push(data.clone());
        debug!(
            "data: {:?}. Total: {}, counter: {}",
            data,
            new_values.len(),
            counter
        );
        counter += 1;
        if counter % FLUSH_SIZE == 0 {
            save2file(file_name, new_values.clone());
            new_values.clear();
            info!("Processed records: {}", counter);
        }
    }
    info!("Processed records: {}", counter);
    info!("new records: {}", new_values.len());
    save2file(file_name, new_values);
}

pub async fn save<T: Clone + Debug + serde::Serialize>(file_name: &str, input: &mut Receiver<T>) {
    let stream = Box::pin(to_stream(input));
    futures::pin_mut!(stream);
    let mut counter = 0;
    let mut new_values = vec![];
    while let Some(data) = stream.next().await {
        new_values.push(data.clone());
        debug!(
            "data: {:?}. Total: {}, counter: {}",
            data,
            new_values.len(),
            counter
        );
        counter += 1;
        if counter % FLUSH_SIZE == 0 {
            save2file(file_name, new_values.clone());
            new_values.clear();
            info!("Processed records: {}", counter);
        }
    }
    info!("Processed records: {}", counter);
    info!("new records: {}", new_values.len());
    save2file(file_name, new_values);
}

pub async fn log_report(log_consumer: &mut Receiver<String>) {
    let stream = Box::pin(to_stream(log_consumer));
    futures::pin_mut!(stream);
    while let Some(log) = stream.next().await {
        info!("log: {}", log);
    }
}

pub async fn print_stream(rx: &mut Receiver<String>) {
    let stream = Box::pin(to_stream(rx));
    futures::pin_mut!(stream);
    while let Some(payload) = stream.next().await {
        info!("payload: {}", payload);
    }
}

//
#[cfg(test)]
mod node_tests {

    use crate::{
        services::mobile_bg_scraper::start_searches,
        utils::helpers::{configure_log4rs, crossbeam_utils::to_stream},
    };
    use crossbeam_channel::Receiver;
    use futures::StreamExt;
    use log::info;

    #[tokio::test]
    async fn ping_pong_test() {}

    #[tokio::test]
    async fn test_searches() {
        configure_log4rs("config/loggers/dev_log4rs.yml");
        let (tx, mut rx) = crossbeam::channel::unbounded::<String>();
        let task = tokio::spawn(async move {
            start_searches(tx).await;
        });
        task.await.unwrap();
        print(&mut rx).await;
    }
    async fn print(rx: &mut Receiver<String>) {
        let stream = Box::pin(to_stream(rx));
        futures::pin_mut!(stream);
        while let Some(payload) = stream.next().await {
            info!("payload: {}", payload);
        }
    }
}
