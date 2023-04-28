use std::collections::HashMap;

use data_scraper::config::links::Mobile;
use data_scraper::model::list::MobileList;
use data_scraper::model::meta::MetaHeader;
use data_scraper::services::PageProcessor::ListProcessor;
use data_scraper::{config_files, configure_log4rs};

use futures::future::{self, FutureExt};
use futures::stream::{self, StreamExt};
use tokio::sync::mpsc;
use tokio::task::block_in_place;

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    configure_log4rs();
    let mobile_config = Mobile::from_file("config/mobile_config.yml");
    config_files::<MobileList>(&mobile_config.config);

    let mut tasks = Vec::new();
    let mut receivers = vec![];
    {
        for config in mobile_config.config {
            for link in config.links {
                let (tx, rx) = mpsc::channel::<HashMap<String, String>>(10);
                receivers.push(rx);
                let metadata = MetaHeader::from_slink(&link.link);
                let processor = ListProcessor::new(
                    link.link.clone(),
                    config.dealear_type.clone(),
                    metadata.total_number,
                    link.filter,
                    tx.clone(),
                );
                tasks.push(processor.start_producer().boxed());
            }
        }

        let task_futures = stream::iter(tasks).map(|t| rt.spawn(t));
        block_in_place(|| {
            rt.block_on(async {
                let handles = task_futures.collect::<Vec<_>>().await;
                future::join_all(handles).await;
            });
        });
    }
}
