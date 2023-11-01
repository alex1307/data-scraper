use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crossbeam_channel::{Receiver, Sender};
use futures::StreamExt;
use log::{info, error, debug};
use tokio::{sync::Mutex, task::JoinHandle};

use crate::{
    model::{records::MobileRecord, search_metadata::asearches},
    scraper::agent::{details2map, get_links},
    utils::{configure_log4rs, crossbeam_utils::to_stream, mobile_search_url, create_empty_csv},
    writer::persistance::{MobileData, MobileDataWriter},
    CONFIG, CREATED_ON, INIT_LOGGER, LISTING_URL,
};
use lazy_static::lazy_static;

use super::file_processor::{self, DataProcessor};
lazy_static! {
    static ref LISTING_MUTEX: Mutex<()> = Mutex::new(());
    static ref DETAILS_MUTEX: Mutex<()> = Mutex::new(());
}

async fn spawn_sequentially(url: String, sender: Sender<String>) -> JoinHandle<()> {
    let _guard = LISTING_MUTEX.lock().await;
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    info!("spawn_sequentially");
    tokio::spawn(async move {
        let links = get_links(url.as_str()).await;
        for link in links {
            info!("sending link{}", link);
            sender.send(link).unwrap();
        }
    })
}

pub async fn start_searches(link_producer: Sender<String>) {
    let logger_file_name = format!("{}/meta_log4rs.yml", CONFIG.get_log4rs_config());
    let metadata_data_file_name = format!("{}/meta_data.csv", CONFIG.get_data_dir());
    INIT_LOGGER.call_once(|| configure_log4rs(&logger_file_name));

    let mut all = vec![];
    let searches = asearches().await;
    //let statistics = astatistic().await;
    info!("searches: {:?}", searches.len());
    //info!("stat: {:?}", statistics.len());
    all.extend(searches.clone());
    //all.extend(statistics.clone());
    for meta in all.iter() {
        info!("{:?}", meta.clone());
    }
    // let mut meta_data_processor: DataProcessor<SearchMetadata> =
    //     file_processor::DataProcessor::from_files(vec![metadata_data_file_name.as_str()]);
    // meta_data_processor.process(&all, None);
    let mut tasks = Vec::new();
    for search in searches.iter() {
        //let pages: Vec<String> = (1..=search.page_numbers()).map(|n| n.to_string()).collect();
        let pages = vec![1.to_string(), 2.to_string()];
        for page in pages {
            info!("page: {}", page);
            let url = mobile_search_url(
                &LISTING_URL.to_string(),
                &page,
                &search.slink,
                crate::model::enums::Dealer::ALL,
                crate::model::enums::SaleType::NONE,
            );
            let task = spawn_sequentially(url, link_producer.clone());
            tasks.push(task);
        }
    }
    for task in tasks {
        task.await;
        info!("task completed");
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

pub async fn process_links(input: &mut Receiver<String>, output: Sender<HashMap<String, String>>) {
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
    let new_recoords_ids: Vec<&String> = new_ids.difference(&archived_ids).collect::<Vec<&String>>();
    let deleted: Vec<&String> = archived_ids.difference(&new_ids).collect::<Vec<&String>>();
    for record in target.iter_mut() {
        if deleted.contains(&&record.id) && record.deleted_on.is_empty() {
            record.deleted_on = CREATED_ON.to_string();
        }else if !new_recoords_ids.contains(&&record.id) {
               record.updated_on = CREATED_ON.to_string();
        }
    }
}

fn save2file(file_name: &str, data: &Vec<MobileRecord>) {
    std::fs::remove_file(&file_name).unwrap();
    if let Err(_) = create_empty_csv::<MobileRecord>(&file_name) {
        error!("Failed to create file {}", file_name);
    }
    let new_data: MobileData<MobileRecord> = MobileData::Payload(data.clone());
    new_data.write_csv(&file_name, false).unwrap();
}

pub async fn save_active_adverts(
    input: &mut Receiver<HashMap<String, String>>,
    output: Sender<String>,
) {
    let stream = Box::pin(to_stream(input));
    futures::pin_mut!(stream);
    let new_details_file_name = format!("{}/vehicle-{}.csv", CONFIG.get_data_dir(), CREATED_ON.to_string());
    let details_file_name = format!("{}/vehicle.archive.csv", "/Users/matkat/Software/Rust/data-scraper/resources/data".to_string());
    let existing_records: DataProcessor<MobileRecord> =
        file_processor::DataProcessor::from_files(vec![&details_file_name]);
    let ids = existing_records.get_ids().clone();
    let mut values = existing_records.get_values().clone();
    let created_on_map: HashMap<String, String> = values
        .iter()
        .map(|mobile| (mobile.id.clone(), mobile.created_on.clone()))
        .collect();
    info!("existing map: {}", created_on_map.len());
    info!("map: {:?}", created_on_map.clone());
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
    }
    info!("Processed records: {}", counter);
    info!("new records: {}", new_values.len());
    info!("archived records: {}", values.len());
    info!("active: {}", new_ids.len());
    info!("not active: {}", ids.difference(&new_ids).count());
    update_archive_records(&new_ids, &ids, &mut values);
    save2file(&details_file_name, &values);
    save2file(&new_details_file_name, &new_values);

}

// pub fn save(consumer: &mut Receiver<Box<dyn Identity>>) {

// }

pub async fn log_report(log_consumer: &mut Receiver<String>) {
    let stream = Box::pin(to_stream(log_consumer));
    futures::pin_mut!(stream);
    while let Some(log) = stream.next().await {
        info!("log: {}", log);
    }
}

//process input from one receiver and send it to the output channel
pub async fn process_input(input: Receiver<String>, output: Sender<Vec<String>>) {
    while let Ok(value) = input.recv() {
        info!("Received value: {}", value);
        if value.to_uppercase() == "STOP".to_string() {
            break;
        }
        let mut values = vec![];
        for i in 0..10 {
            values.push(format!("{}-{}", value, i));
        }
        output.send(values).unwrap();
    }
}

pub async fn process_input1(
    input: Receiver<Vec<String>>,
    output: Sender<String>,
    shutdown: Arc<AtomicBool>,
) {
    let mut counter = 0;
    while let Ok(value) = input.recv() {
        counter += value.len();
        if counter > 100 || shutdown.load(Ordering::Relaxed) {
            info!("counter: {}", counter);
            info!("Shutting down...{}", shutdown.load(Ordering::Relaxed));
            break;
        }
    }
    output.send(counter.to_string()).unwrap();
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
    use std::sync::{atomic::AtomicBool, Arc};
    use std::vec;

    use crossbeam_channel::Receiver;
    use futures::StreamExt;
    use log::info;
    
    use crate::services::node::process_input1;
    use crate::services::node::{process_input, start_searches};
    use crate::utils::configure_log4rs;
    use crate::utils::crossbeam_utils::to_stream;
    

    #[tokio::test]
    async fn ping_pong_test() {
        configure_log4rs("config/loggers/dev_log4rs.yml");
        let (tx1, rx1) = crossbeam::channel::unbounded::<String>();
        let (tx2, rx2) = crossbeam::channel::unbounded::<Vec<String>>();
        let (tx3, rx3) = crossbeam::channel::unbounded::<String>();
        let w1 = tokio::spawn(async move {
            process_input(rx1, tx2).await;
        });
        tx1.send("ping".to_string()).unwrap();
        tx1.send("stop".to_string()).unwrap();
        let mut counter = 0;
        let shutdown = Arc::new(AtomicBool::new(false));
        let w2 = tokio::spawn(async move {
            process_input1(rx2, tx3.clone(), shutdown).await;
        });

        let workers = vec![w1, w2];
        for w in workers {
            w.await.unwrap();
        }

        while let Ok(value) = rx3.recv() {
            counter += value.parse::<u32>().unwrap();
            info!("counter: {}", counter);
            assert_eq!(counter, 10);
        }
    }

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
