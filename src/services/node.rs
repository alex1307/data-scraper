use std::{
    collections::{HashMap, HashSet},
    error::Error,
};

use crossbeam_channel::{Receiver, Sender};
use futures::StreamExt;
use log::{debug, error, info};
use tokio::{sync::Mutex, task::JoinHandle};

use crate::{
    model::{
        records::MobileRecord,
        search_metadata::{asearches, SearchMetadata},
    },
    scraper::agent::{details2map, get_links},
    utils::{create_empty_csv, crossbeam_utils::to_stream, mobile_search_url},
    writer::persistance::{MobileData, MobileDataWriter},
    ARCHIVE_FILE_NAME, CREATED_ON, INSALE_FILE_NAME, LISTING_URL, METADATA_FILE_NAME,
};
use lazy_static::lazy_static;

use super::file_processor::{self, DataProcessor};
lazy_static! {
    static ref LISTING_MUTEX: Mutex<()> = Mutex::new(());
    static ref DETAILS_MUTEX: Mutex<()> = Mutex::new(());
}

pub async fn scrape() -> Result<(), Box<dyn Error>> {
    if let Err(_) = create_empty_csv::<MobileRecord>(&INSALE_FILE_NAME) {
        error!("Failed to create file {}", INSALE_FILE_NAME.clone());
    }

    if let Err(_) = create_empty_csv::<MobileRecord>(&ARCHIVE_FILE_NAME) {
        error!("Failed to create file {:?}", ARCHIVE_FILE_NAME.clone());
    }

    if let Err(_) = create_empty_csv::<SearchMetadata>(&METADATA_FILE_NAME) {
        error!("Failed to create file {:?}", METADATA_FILE_NAME.clone());
    }

    let (link_producer, mut link_consumer) = crossbeam::channel::unbounded::<String>();
    let (details_producer, mut details_consumer) =
        crossbeam::channel::unbounded::<HashMap<String, String>>();
    let task = tokio::spawn(async move {
        start_searches(link_producer).await;
    });
    let task2 = tokio::spawn(async move {
        process_links(&mut link_consumer, details_producer).await;
    });
    let task3 = tokio::spawn(async move {
        save_active_adverts(&mut details_consumer).await;
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
        for link in links {
            sender.send(link).unwrap();
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
                &LISTING_URL.to_string(),
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

async fn process_links(input: &mut Receiver<String>, output: Sender<HashMap<String, String>>) {
    let stream = Box::pin(to_stream(input));
    futures::pin_mut!(stream);
    let mut counter = 0;
    while let Some(url) = stream.next().await {
        debug!("url: {}", url.clone());
        let data = details2map(url.as_str()).await;
        if data.is_empty() {
            continue;
        }
        output.send(data).unwrap();
        //sleep for 100 millis
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        counter += 1;
    }
    info!("Processed urls: {}", counter);
}

fn create_record(
    record: &mut MobileRecord,
    created_on_map: &HashMap<String, String>,
    new_ids: &mut HashSet<String>,
    new_records: &mut Vec<MobileRecord>,
    archived_records: &mut Vec<MobileRecord>,
) {
    if let Some(created_on) = created_on_map.get(&record.id) {
        record.created_on = created_on.to_string();
        record.updated_on = CREATED_ON.to_string();
    } else {
        record.created_on = CREATED_ON.to_string();
        archived_records.push(record.clone());
    }
    new_records.push(record.clone());
    new_ids.insert(record.id.clone());
}

fn update_archive_records(
    new_ids: &HashSet<String>,
    archived_ids: &HashSet<String>,
    target: &mut Vec<MobileRecord>,
) {
    let new_recoords_ids: Vec<&String> =
        new_ids.difference(&archived_ids).collect::<Vec<&String>>();
    let deleted: Vec<&String> = archived_ids.difference(&new_ids).collect::<Vec<&String>>();
    for record in target.iter_mut() {
        if deleted.contains(&&record.id) && record.deleted_on.is_empty() {
            record.deleted_on = CREATED_ON.to_string();
        } else if !new_recoords_ids.contains(&&record.id) {
            record.updated_on = CREATED_ON.to_string();
        }
    }
}

fn save2file(file_name: &str, data: &Vec<MobileRecord>) {
    if let Err(_) = std::fs::remove_file(&file_name) {
        error!("Failed to remove file {}", file_name);
    }
    if let Err(_) = create_empty_csv::<MobileRecord>(&file_name) {
        error!("Failed to create file {}", file_name);
    }
    let new_data: MobileData<MobileRecord> = MobileData::Payload(data.clone());
    new_data.write_csv(&file_name, false).unwrap();
}

pub async fn save_active_adverts(input: &mut Receiver<HashMap<String, String>>) {
    let stream = Box::pin(to_stream(input));
    futures::pin_mut!(stream);
    let existing_records: DataProcessor<MobileRecord> =
        file_processor::DataProcessor::from_files(vec![&ARCHIVE_FILE_NAME]);
    let ids = existing_records.get_ids().clone();
    let mut values = existing_records.get_values().clone();
    let created_on_map: HashMap<String, String> = values
        .iter()
        .map(|mobile| (mobile.id.clone(), mobile.created_on.clone()))
        .collect();
    let mut counter = 0;
    let mut new_values = vec![];
    let mut new_ids = HashSet::new();
    while let Some(data) = stream.next().await {
        let mut record = MobileRecord::from(data);
        create_record(
            &mut record,
            &created_on_map,
            &mut new_ids,
            &mut new_values,
            &mut values,
        );
        counter += 1;
        if counter % 250 == 0 {
            info!("Processed records: {}", counter);
        }
    }
    info!("Processed records: {}", counter);
    info!("new records: {}", new_values.len());
    info!("archived records: {}", values.len());
    info!("active: {}", new_ids.len());
    info!("not active: {}", ids.difference(&new_ids).count());
    update_archive_records(&new_ids, &ids, &mut values);
    save2file(&ARCHIVE_FILE_NAME, &values);
    save2file(&INSALE_FILE_NAME, &new_values);
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

    use crossbeam_channel::Receiver;
    use futures::StreamExt;
    use log::info;

    use crate::services::node::start_searches;
    use crate::utils::configure_log4rs;
    use crate::utils::crossbeam_utils::to_stream;

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
