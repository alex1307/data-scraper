use std::{collections::HashSet, error::Error, vec, fmt::Debug};

use crossbeam_channel::{Receiver, Sender};
use futures::StreamExt;
use log::{debug, error, info};
use reqwest::Url;
use tokio::{sync::Mutex, task::JoinHandle};

use crate::{
    model::{
        records::MobileRecord,
        search_metadata::{asearches, SearchMetadata}, id_list::IDList, traits::Identity,
    },
    scraper::mobile_bg::{details2map, get_links},
    utils::helpers::{create_empty_csv, crossbeam_utils::to_stream, mobile_search_url},
    writer::persistance::{MobileData, MobileDataWriter},
    ARCHIVE_FILE_NAME, INSALE_FILE_NAME, LISTING_URL, METADATA_FILE_NAME, FOR_UPDATE_FILE_NAME, DETAILS_URL, UPDATED_FILE_NAME, DELETED_FILE_NAME,
};
use lazy_static::lazy_static;

use super::file_processor::{self, DataProcessor};
pub const FLUSH_SIZE: usize = 50;
lazy_static! {
    static ref LISTING_MUTEX: Mutex<()> = Mutex::new(());
    static ref DETAILS_MUTEX: Mutex<()> = Mutex::new(());
    static ref UNIQUE_IDS: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

pub async fn update() ->Result<(), Box<dyn Error>> {
    
    let update_processor = DataProcessor::<IDList>::from_files(vec![&FOR_UPDATE_FILE_NAME]);
    let update_data = update_processor.get_ids().clone();
    let cloned_data = update_data.clone();
    let search_all = SearchMetadata::search(crate::model::enums::SaleType::NONE, 1, 9_999_999);
    let mut urls = HashSet::new();
    
    let (link_producer, mut link_consumer) = crossbeam::channel::unbounded::<String>();
    let (validate_producer, mut validate_consumer) = crossbeam::channel::unbounded::<MobileRecord>();
    let (record_producer, mut record_consumer) = crossbeam::channel::unbounded::<MobileRecord>();
    let (confirming_producer, mut confirming_consumer) = crossbeam::channel::unbounded::<String>();
    
    let producer_task = tokio::spawn(async move {
        for id in update_data {
            let url= format!("{}&adv={}&slink={}", DETAILS_URL, id, search_all.slink.clone());
            link_producer.send(url.clone()).unwrap();
            urls.insert(url);
        }
    });
    
    let process_links_task = tokio::spawn(async move {
        process_links(&mut link_consumer, validate_producer).await;
    });

    let confirm_task = tokio::spawn(async move {
        confirm(&mut validate_consumer, confirming_producer, record_producer).await;
    });

    let save_to_file_task = tokio::spawn(async move {
        save(&INSALE_FILE_NAME, &mut record_consumer).await;
    });

    let update_task = tokio::spawn(async move {
        let stream = Box::pin(to_stream(&mut confirming_consumer));
        futures::pin_mut!(stream);
        let mut updated = vec![];
        let mut counter = 0;
        while let Some(record) = stream.next().await {
            let id = IDList::new(record.clone());
            updated.push(id);
            counter += 1;
        }
        save2file(&UPDATED_FILE_NAME, updated.clone());
        let ids = updated.iter().map(|l| l.get_id()).collect::<HashSet<String>>();
        let deleted = cloned_data.difference(&ids);
        let mut deleted_vec = vec![];
        for id in deleted {
            deleted_vec.push(IDList::new(id.clone()));
        }
        save2file(&DELETED_FILE_NAME, deleted_vec);
        info!("Processed records: {}", counter);

    });

    if let (Ok(_), Ok(_), Ok(_), Ok(_)) = tokio::join!(
        producer_task,
        process_links_task, 
        confirm_task, 
        save_to_file_task) {
        info!("All tasks completed successfully");
        
        

        Ok(())
    } else {
        error!("One or more tasks failed");
        Err("One or more tasks failed".into())
    }
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
    let (details_producer, mut details_consumer) = crossbeam::channel::unbounded::<MobileRecord>();
    let task = tokio::spawn(async move {
        start_searches(link_producer).await;
    });
    let task2 = tokio::spawn(async move {
        process_links(&mut link_consumer, details_producer).await;
    });
    let task3 = tokio::spawn(async move {
        save(&INSALE_FILE_NAME, &mut details_consumer).await;
    });

    if let (Ok(_), Ok(_), Ok(_)) = tokio::join!(task, task2, task3) {
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
        let links = get_links(url.as_str()).await;
        let mut ids = UNIQUE_IDS.lock().await;
        for link in links {
            let url = Url::parse(&link).expect("Failed to parse URL");
            if let Some(adv_value) = url.query_pairs().find(|(key, _)| key == "adv") {
                if ids.contains(&adv_value.1.to_string()) {
                    continue;
                } else {
                    ids.insert(adv_value.1.to_string());
                    sender.send(link).unwrap();
                }
            }
        }
    })
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
    UNIQUE_IDS.lock().await.clear();
}

async fn process_links(input: &mut Receiver<String>, output: Sender<MobileRecord>) {
    let stream = Box::pin(to_stream(input));
    futures::pin_mut!(stream);
    let mut counter = 0;
    while let Some(url) = stream.next().await {
        debug!("url: {}", url.clone());
        let data = details2map(url.as_str()).await;
        if data.is_empty() {
            continue;
        }
        let record = MobileRecord::from(data);
        output.send(record).unwrap();
        //sleep for 100 millis
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        counter += 1;
    }
    info!("Processed urls: {}", counter);
}

async fn confirm(input: &mut Receiver<MobileRecord>, confirmed: Sender<String>, forward: Sender<MobileRecord>) {
    let stream = Box::pin(to_stream(input));
    futures::pin_mut!(stream);
    while let Some(record) = stream.next().await {
        confirmed.send(record.id.clone()).unwrap();
        forward.send(record).unwrap();
    }
}

fn save2file<T:Clone + Debug + serde::Serialize>(file_name: &str, data: Vec<T>) {
    // if let Err(_) = std::fs::remove_file(&file_name) {
    //     error!("Failed to remove file {}", file_name);
    // }
    // if let Err(_) = create_empty_csv::<MobileRecord>(&file_name) {
    //     error!("Failed to create file {}", file_name);
    // }
    let new_data = MobileData::Payload(data);
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

pub async fn save<T:Clone + Debug + serde::Serialize>(file_name: &str ,input: &mut Receiver<T>) {
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
            save2file(&INSALE_FILE_NAME, new_values.clone());
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