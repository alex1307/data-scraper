use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt::Debug,
    vec,
};

use crossbeam_channel::{Receiver, Sender};
use futures::StreamExt;
use log::{debug, error, info};
use reqwest::Url;
use tokio::{sync::Mutex, task::JoinHandle};

lazy_static! {
    pub static ref CARS_BG_INSALE_FILE_NAME: String = format!(
        "{}/cars-bg-vehicle-{}.csv",
        CONFIG.get_data_dir(),
        CREATED_ON.clone()
    );
    pub static ref CARS_BG_UPDATED_VEHICLES_FILE_NAME: String = format!(
        "{}/cars-bg-updated-vehicle-{}.csv",
        CONFIG.get_data_dir(),
        CREATED_ON.clone()
    );
    pub static ref CARS_BG_METADATA_FILE_NAME: String =
        format!("{}/cars-meta-data.csv", CONFIG.get_data_dir());
    pub static ref CARS_BG_FOR_UPDATE_FILE_NAME: String =
        format!("{}/cars-bg-ids-for-update.csv", CONFIG.get_data_dir());
    pub static ref CARS_BG_DELETED_FILE_NAME: String = format!(
        "{}/not-found-ids-{}.csv",
        CONFIG.get_data_dir(),
        CREATED_ON.clone()
    );
}

use crate::{
    model::{
        enums::SaleType,
        id_list::IDList,
        records::MobileRecord,
        search_metadata::{asearch, asearches, SearchMetadata},
    },
    scraper::{
        cars_bg::search_cars_bg,
        mobile_bg::{details2map, get_links},
    },
    utils::helpers::{create_empty_csv, crossbeam_utils::to_stream, mobile_search_url},
    writer::persistance::{MobileData, MobileDataWriter},
    CONFIG, CREATED_ON,
};
use lazy_static::lazy_static;

use super::file_processor::{self, DataProcessor};
pub const FLUSH_SIZE: usize = 400;
lazy_static! {
    static ref LISTING_MUTEX: Mutex<()> = Mutex::new(());
    static ref DETAILS_MUTEX: Mutex<()> = Mutex::new(());
}

pub async fn scrape() -> Result<(), Box<dyn Error>> {
    if create_empty_csv::<MobileRecord>(&CARS_BG_INSALE_FILE_NAME).is_err() {
        error!("Failed to create file {}", CARS_BG_INSALE_FILE_NAME.clone());
    }

    if create_empty_csv::<SearchMetadata>(&CARS_BG_INSALE_FILE_NAME).is_err() {
        error!(
            "Failed to create file {:?}",
            CARS_BG_INSALE_FILE_NAME.clone()
        );
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
        save(&CARS_BG_INSALE_FILE_NAME, &mut details_consumer).await;
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
    let prices = vec![
        1000, 2000, 3000, 4000, 5000, 6000, 7000, 8000, 9000, 10_000, 11_000, 13_000, 15_000,
        18_000, 19_000, 21_000, 25_000, 30_000, 40_000, 50_000, 95_000,
    ];
    let mut map = HashMap::new();
    map.insert("subm".to_owned(), "1".to_owned());
    map.insert("add_search".to_owned(), "1".to_owned());
    map.insert("typeoffer".to_owned(), "1".to_owned());
    map.insert("last".to_owned(), "1".to_owned());
    map.insert("conditions[]".to_owned(), "1".to_owned());
    map.insert("yearFrom".to_owned(), "2004".to_owned());
    map.insert("steering_wheel".to_owned(), "1".to_owned());
    let mut searches = vec![];
    for i in 0..prices.len() - 2 {
        let mut price_filter = map.clone();
        price_filter.insert("priceFrom".to_owned(), prices[i].to_string());
        price_filter.insert("priceTo".to_owned(), (prices[i + 1]).to_string());
        searches.push(price_filter);
    }
    let mut over_100K = map.clone();
    over_100K.insert("priceFrom".to_owned(), "95000".to_owned());
    searches.push(over_100K);

    for search in searches {
        search_cars_bg(search).await;
    }

    let mut all = vec![];
    let searches = asearches().await;
    info!("searches: {:?}", searches.len());
    all.extend(searches.clone());
    for meta in all.iter() {
        info!("{:?}", meta.clone());
    }
    let mut meta_data_processor: DataProcessor<SearchMetadata> =
        file_processor::DataProcessor::from_files(vec![&CARS_BG_METADATA_FILE_NAME]);
    meta_data_processor.process(&all, None);
    // let mut tasks = Vec::new();
    let mut counter = 0;
    for search in searches.iter() {
        counter += search.total_number;
        let pages: Vec<String> = (1..=search.page_numbers()).map(|n| n.to_string()).collect();
        //let pages = vec![1.to_string()];
        for page in pages {
            // let url = mobile_search_url(
            //     LISTING_URL,
            //     &page,
            //     &search.slink,
            //     crate::model::enums::SaleType::NONE,
            //     0,
            //     0,
            // );
            // let task = spawn_sequentially(url, link_producer.clone());
            // tasks.push(task);
        }
    }
    info!("Total number of links: {}", counter);

    // for task in tasks {
    //     task.await;
    // }
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
                for u in &urls {
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

    use crossbeam_channel::Receiver;
    use futures::StreamExt;
    use log::info;

    use crate::{
        services::cars_bg_scraper::start_searches,
        utils::helpers::{configure_log4rs, crossbeam_utils::to_stream},
    };

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
