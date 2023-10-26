use std::collections::{HashMap, HashSet};

use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use futures::executor::block_on;
use futures::future::{self, FutureExt};
use futures::stream::{self, StreamExt};
use log::{error, info};

use tokio::spawn;
use tokio::task::block_in_place;

use crate::config::app_config::AppConfig;
use crate::config::links::Mobile;
use crate::model::details::MobileDetails;
use crate::model::enums::Payload;
use crate::model::error::DataError;
use crate::model::list::MobileList;
use crate::model::meta::searches;
use crate::services::file_processor;
use crate::services::stream_processor::process;
use crate::services::streamer::DataStream;
use crate::utils::{configure_log4rs, create_empty_csv, get_file_names};
use crate::{DATE_FORMAT, DETAILS_URL, LISTING_URL};

pub async fn scrape_details(slink: &str) {
    let app_config = AppConfig::from_file("config/config.yml");
    let logger_file_name = format!("{}/details_log4rs.yml", app_config.get_log4rs_config());
    let source_data_file_name = format!("{}/listing.csv", app_config.get_data_dir());
    let scrpaer_config_file = app_config.get_scraper_config();
    let created_on = chrono::Utc::now().format(DATE_FORMAT).to_string();
    let details_file_name = format!("{}/details_{}.csv", app_config.get_data_dir(), created_on);
    let errors_file_name = format!("{}/errors_{}.csv", app_config.get_data_dir(), created_on);
    let pattern = format!("{}/errors_", app_config.get_data_dir());
    let error_files_name = get_file_names(&pattern, "2023-05-30", "", "csv");
    let files: Vec<&str> = error_files_name.iter().map(|f| f.as_str()).collect();
    configure_log4rs(&logger_file_name);
    info!("----------------------------------------");
    info!("Starting DETAILS application on {}", created_on);
    info!("scraper config file: {}", scrpaer_config_file);
    info!("target file: {}", details_file_name);
    info!("source data file: {}", source_data_file_name);
    info!("number of threads: {}", app_config.get_num_threads());
    info!("----------------------------------------");
    let processor: file_processor::DataProcessor<MobileList> =
        file_processor::DataProcessor::from_files(vec![&source_data_file_name]);
    let error_processor: file_processor::DataProcessor<DataError> =
        file_processor::DataProcessor::from_files(files);
    let source_ids = processor.get_ids();
    let error_ids = error_processor.get_ids();
    let details_processor: file_processor::DataProcessor<MobileDetails> =
        file_processor::DataProcessor::from_files(vec![&details_file_name]);
    let details_ids = details_processor.get_ids();
    let union_ids = details_ids
        .union(error_ids)
        .cloned()
        .collect::<HashSet<String>>();
    let ids = source_ids
        .difference(&union_ids)
        .cloned()
        .collect::<Vec<String>>();
    info!("Number of ids to process: {}", ids.len());
    let chunk_size = ids.len() / app_config.get_num_threads();
    let chunks = ids
        .chunks(chunk_size)
        .map(|c| c.to_vec())
        .collect::<Vec<_>>();
    let mobile_config = Mobile::from_file(scrpaer_config_file);
    let mut tasks = Vec::new();
    let (tx, mut rx) = crossbeam::channel::unbounded::<Payload<HashMap<String, String>>>();
    let (etx, mut erx) = crossbeam::channel::unbounded::<Payload<HashMap<String, String>>>();
    let mut counter = Arc::new(AtomicUsize::new(0));
    let mut counter2 = Arc::new(AtomicUsize::new(0));
    {
        let found = mobile_config
            .config
            .iter()
            .find(|cfg| cfg.dealear_type == "ALL")
            .and_then(|cfg| cfg.links.iter().find(|link| link.name == "ALL"));

        let _link = if found.is_some() {
            found.unwrap()
        } else {
            error!("No link found");
            return;
        };
        for chunk in chunks {
            let mut processor = DataStream::new(
                DETAILS_URL.to_owned(),
                slink.to_string(),
                "".to_owned(),
                chunk,
                tx.clone(),
            );
            processor.with_error_handler(etx.clone());
            counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            counter2.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            tasks.push(async move { processor.stream().await }.boxed());
        }

        tasks.push(
            async move {
                let _ = create_empty_csv::<MobileDetails>(&details_file_name);
                process::<MobileDetails>(&mut rx, &details_file_name, &mut counter).await
            }
            .boxed(),
        );

        tasks.push(
            async move {
                let _ = create_empty_csv::<DataError>(&errors_file_name);
                process::<DataError>(&mut erx, &errors_file_name, &mut counter2).await
            }
            .boxed(),
        );

        let task_futures = stream::iter(tasks).map(spawn);
        block_in_place(|| {
            block_on(async {
                let handles = task_futures.collect::<Vec<_>>().await;
                future::join_all(handles).await;
            });
        });
    }
}

pub async fn scrape_listing() {
    let app_config = AppConfig::from_file("config/config.yml");
    let logger_file_name = format!("{}/listing_log4rs.yml", app_config.get_log4rs_config());
    let listing_data_file_name = format!("{}/listing.csv", app_config.get_data_dir());
    let scrpaer_config_file = app_config.get_scraper_config();
    let created_on = chrono::Utc::now().format(DATE_FORMAT).to_string();

    configure_log4rs(&logger_file_name);
    info!("----------------------------------------");
    info!("Starting *LISTING* application on {}", created_on);
    info!("scraper config file: {}", scrpaer_config_file);
    info!("listing data file: {}", listing_data_file_name);
    info!("number of threads: {}", app_config.get_num_threads());
    info!("----------------------------------------");
    let mobile_config = Mobile::from_file(scrpaer_config_file);
    info!("Config {:#?}", mobile_config);
    let mut tasks = Vec::new();
    let (tx, mut rx) = crossbeam::channel::unbounded::<Payload<HashMap<String, String>>>();
    let mut counter = Arc::new(AtomicUsize::new(0));
    let searches = searches();
    for search in searches {
        let pages: Vec<String> = (1..=search.page_numbers()).map(|n| n.to_string()).collect();
        let mut processor = DataStream::new(
            LISTING_URL.to_string(),
            search.slink,
            search.dealer.to_string(),
            pages.clone(),
            tx.clone(),
        );
        counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        tasks.push(async move { processor.stream().await }.boxed());
    }

    info!(
        "Total tasks: {}, counter: {}",
        tasks.len(),
        counter.load(std::sync::atomic::Ordering::SeqCst)
    );

    tasks.push(
        async move { process::<MobileList>(&mut rx, &listing_data_file_name, &mut counter).await }
            .boxed(),
    );
    let task_futures = stream::iter(tasks).map(spawn);
    block_in_place(|| {
        block_on(async {
            let handles = task_futures.collect::<Vec<_>>().await;
            future::join_all(handles).await;
        });
    });
}
