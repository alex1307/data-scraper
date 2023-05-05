use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use data_scraper::config::links::Mobile;
use data_scraper::model::enums::Payload;
use data_scraper::model::list::MobileList;
use data_scraper::model::meta::MetaHeader;

use data_scraper::services::stream_processor::process;
use data_scraper::services::streamer::DataStream;
use data_scraper::utils::{config_files, configure_log4rs};

use futures::future::{self, FutureExt};
use futures::stream::{self, StreamExt};
use log::info;
use tokio::task::block_in_place;

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    configure_log4rs();
    let mobile_config = Mobile::from_file("config/mobile_config.yml");
    info!("Config {:#?}", mobile_config);
    config_files::<MobileList>(&mobile_config.config);
    let mut tasks = Vec::new();
    let (tx, mut rx) = crossbeam::channel::unbounded::<Payload<HashMap<String, String>>>();
    let mut counter = Arc::new(AtomicUsize::new(0));
    {
        for config in mobile_config.config {
            for link in config.links {
                if link.scrape == false {
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
                process::<MobileList>(&mut rx, "resources/data/listing.csv", &mut counter).await
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
