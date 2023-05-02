use std::collections::HashMap;

use data_scraper::config::links::Mobile;
use data_scraper::model::list::MobileList;
use data_scraper::model::meta::MetaHeader;
use data_scraper::services::FileProcessor;
use data_scraper::services::PageProcessor::ListProcessor;
use data_scraper::utils::crossbeam_utils::to_stream;
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
    let (tx, mut rx) = crossbeam::channel::unbounded::<HashMap<String, String>>();
    {
        for config in mobile_config.config {
            for link in config.links {
                if link.scrape == false {
                    info!("Skipping {:#?}, {}", &link.name, &link.link);
                    continue;
                }
                let metadata = MetaHeader::from_slink(&link.link);
                let processor = ListProcessor::new(
                    link.link.clone(),
                    link.name.clone(),
                    config.dealear_type.clone(),
                    metadata.page_numbers(),
                    link.filter,
                    tx.clone(),
                );
                tasks.push(processor.start_producer().boxed());
            }
        }
        tasks.push(
            async move {
                let stream = Box::pin(to_stream(&mut rx));
                let mut counter = 0;

                let mut processor: FileProcessor::DataProcessor<MobileList> =
                    FileProcessor::DataProcessor::from_file("resources/data/listing.csv");
                let mut values = vec![];
                let mut map_counter: HashMap<String, i32> = HashMap::new();
                futures::pin_mut!(stream);
                while let Some(data) = stream.next().await {
                    counter += 1;
                    if let Some(count) = map_counter.get_mut(&data["slink"]) {
                        *count += 1;
                    } else {
                        map_counter.insert(data["slink"].clone(), 1);
                    }
                    let value = MobileList::from(data);
                    values.push(MobileList::from(value));
                    if values.len() >= 20 {
                        info!("Sending {:?}", values.len());
                        processor.process(&values, None);
                        values = vec![];
                        info!("Statistic {:#?}", map_counter);
                    }
                }
                if !values.is_empty() {
                    processor.process(&values, None);
                }
                info!("Total Data {:?}", counter);
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
