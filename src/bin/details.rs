use std::collections::HashMap;

use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use data_scraper::config::app_config::AppConfig;
use data_scraper::config::links::Mobile;
use data_scraper::model::details::MobileDetails;
use data_scraper::model::enums::Payload;
use data_scraper::model::list::MobileList;

use data_scraper::services::file_processor;
use data_scraper::services::stream_processor::process;
use data_scraper::services::streamer::DataStream;

use data_scraper::utils::{config_files, configure_log4rs, create_empty_csv};

use futures::executor::block_on;
use futures::future::{self, FutureExt};
use futures::stream::{self, StreamExt};
use log::{error, info};
use tokio::spawn;
use tokio::task::block_in_place;

#[tokio::main]
async fn main() {
    let app_config = AppConfig::from_file("config/config.yml");
    let logger_file_name = format!("{}/details_log4rs.yml", app_config.get_log4rs_config());
    let source_data_file_name = format!("{}/listing.csv", app_config.get_data_dir());
    let scrpaer_config_file = app_config.get_scraper_config();
    let created_on = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let details_file_name = format!("{}/details_{}.csv", app_config.get_data_dir(), created_on);

    configure_log4rs(&logger_file_name);
    info!("----------------------------------------");
    info!("Starting DETAILS application on {}", created_on);
    info!("scraper config file: {}", scrpaer_config_file);
    info!("target file: {}", details_file_name);
    info!("source data file: {}", source_data_file_name);
    info!("number of threads: {}", app_config.get_num_threads());
    info!("----------------------------------------");
    let processor: file_processor::DataProcessor<MobileList> =
        file_processor::DataProcessor::from_file(&source_data_file_name);
    let ids: Vec<String> = processor.get_ids().iter().cloned().collect();

    let chunk_size = ids.len() / app_config.get_num_threads();
    let chunks = ids
        .chunks(chunk_size)
        .map(|c| c.to_vec())
        .collect::<Vec<_>>();
    let mobile_config = Mobile::from_file(&scrpaer_config_file);
    config_files::<MobileDetails>(&mobile_config.config);
    let mut tasks = Vec::new();
    let (tx, mut rx) = crossbeam::channel::unbounded::<Payload<HashMap<String, String>>>();
    let mut counter = Arc::new(AtomicUsize::new(0));
    {
        let found = mobile_config
            .config
            .iter()
            .find(|cfg| cfg.dealear_type == "ALL")
            .and_then(|cfg| {
                cfg.links
                    .iter()
                    .find(|link| link.name == "ALL")
                    .map(|link| link.clone())
            });

        let link = if found.is_some() {
            found.unwrap()
        } else {
            error!("No link found");
            return;
        };
        for chunk in chunks {
            let mut processor = DataStream::new(link.clone(), chunk, tx.clone());
            counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            tasks.push(async move { processor.stream().await }.boxed());
        }

        tasks.push(
            async move {
                let _ = create_empty_csv::<MobileDetails>(&details_file_name);
                process::<MobileDetails>(&mut rx, &details_file_name, &mut counter).await
            }
            .boxed(),
        );
        let task_futures = stream::iter(tasks).map(|t| spawn(t));
        block_in_place(|| {
            block_on(async {
                let handles = task_futures.collect::<Vec<_>>().await;
                future::join_all(handles).await;
            });
        });
    }
}
