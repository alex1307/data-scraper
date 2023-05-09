use std::collections::HashMap;

use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

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
use log::error;
use tokio::spawn;
use tokio::task::block_in_place;

#[tokio::main]
async fn main() {
    configure_log4rs();
    let processor: file_processor::DataProcessor<MobileList> =
        file_processor::DataProcessor::from_file("resources/data/listing.csv");
    let ids = processor.get_ids().iter().cloned().collect();
    let mobile_config = Mobile::from_file("config/mobile_config.yml");
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
        let mut processor = DataStream::new(link.clone(), ids, tx.clone());
        counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        tasks.push(async move { processor.stream().await }.boxed());
        tasks.push(
            async move {
                let created_on = chrono::Utc::now().format("%Y-%m-%d").to_string();
                let file_name = format!("resources/data/details_{}.csv", created_on);
                let _ = create_empty_csv::<MobileDetails>(&file_name);
                process::<MobileDetails>(&mut rx, &file_name, &mut counter).await
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
