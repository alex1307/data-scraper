use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use data_scraper::config::app_config::AppConfig;
use data_scraper::config::links::Mobile;
use data_scraper::model::enums::Payload;
use data_scraper::model::list::MobileList;
use data_scraper::model::meta::MetaHeader;
use data_scraper::DATE_FORMAT;

use data_scraper::services::stream_processor::process;
use data_scraper::services::streamer::DataStream;
use data_scraper::utils::{config_files, configure_log4rs};

use futures::future::{self, FutureExt};
use futures::stream::{self, StreamExt};
use log::info;
use tokio::task::block_in_place;

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
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
    config_files::<MobileList>(&mobile_config.config);
    let mut tasks = Vec::new();
    let (tx, mut rx) = crossbeam::channel::unbounded::<Payload<HashMap<String, String>>>();
    let mut counter = Arc::new(AtomicUsize::new(0));
    {
        for config in mobile_config.config {
            for link in config.links {
                if !link.scrape {
                    info!("Skipping {:#?}, {}", &link.name, &link.link);
                    continue;
                }
                let metadata = MetaHeader::from_slink(&link.link);
                let pages: Vec<String> = (1..=metadata.page_numbers())
                    .map(|n| n.to_string())
                    .collect();
                let mut processor = DataStream::new(link.clone(), pages, tx.clone());
                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                tasks.push(async move { processor.stream().await }.boxed());
            }
        }

        info!(
            "Total tasks: {}, counter: {}",
            tasks.len(),
            counter.load(std::sync::atomic::Ordering::SeqCst)
        );

        tasks.push(
            async move {
                process::<MobileList>(&mut rx, &listing_data_file_name, &mut counter).await
            }
            .boxed(),
        );
        let task_futures = stream::iter(tasks).map(|t| rt.spawn(t));
        block_in_place(|| {
            rt.block_on(async {
                let handles = task_futures.collect::<Vec<_>>().await;
                future::join_all(handles).await;
            });
        });
    }
}
