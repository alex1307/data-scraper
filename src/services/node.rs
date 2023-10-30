use std::{
    env::var,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crossbeam::thread;
use crossbeam_channel::{Receiver, Sender};
use futures::{stream, FutureExt, StreamExt};
use log::info;
use scraper::Html;
use tokio::spawn;

use crate::{
    config::app_config::AppConfig,
    model::search_metadata::{asearches, astatistic, SearchMetadata},
    scraper::agent::{get_links, get_metadata_links, get_pages_async},
    utils::{configure_log4rs, mobile_search_url, crossbeam_utils::to_stream},
    DATE_FORMAT, LISTING_URL,
};

use super::file_processor::{self, DataProcessor};

pub async fn start_searches(link_producer: Sender<String>) {
    let app_config = AppConfig::from_file("config/config.yml");
    let logger_file_name = format!("{}/meta_log4rs.yml", app_config.get_log4rs_config());
    let metadata_data_file_name = format!("{}/meta_data.csv", app_config.get_data_dir());
    configure_log4rs(&logger_file_name);
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

            let cloned = link_producer.clone();
            let task = tokio::spawn(async move {
                let links = get_links(url.as_str()).await;
                for link in links {
                    info!("sending link{}", link);
                    cloned.send(link).unwrap();
                }
            });
            tasks.push(task);
        }
    }
    for task in tasks {
        task.await.unwrap();
        info!("task completed");
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
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
    use futures::executor::block_on;
    use log::info;
    use tokio::task::block_in_place;

    use crate::services::node::process_input1;
    use crate::services::node::{process_input, start_searches};
    use crate::utils::configure_log4rs;
    use crate::utils::crossbeam_utils::to_stream;
    use futures::future::{self, FutureExt};

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
