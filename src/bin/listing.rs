use std::{collections::HashMap, vec};

use data_scraper::{
    config::links::Mobile,
    model::{list::MobileList, meta::MetaHeader},
    services::PageProcessor::ListProcessor,
    utils::{config_files, configure_log4rs},
};

use log::info;

#[tokio::main]
async fn main() {
    configure_log4rs();
    let mobile_config: Mobile = Mobile::from_file("config/mobile_config.yml");
    info!("Config {:#?}", mobile_config);
    config_files::<MobileList>(&mobile_config.config);
    let mut recievers = vec![];
    let mut tasks = Vec::new();

    {
        for config in mobile_config.config {
            for link in config.links {
                let (tx, rx) = crossbeam_channel::unbounded::<HashMap<String, String>>();
                recievers.push(rx);
                let metadata = MetaHeader::from_slink(&link.link);
                let processor = ListProcessor::new(
                    link.link.clone(),
                    link.name.clone(),
                    config.dealear_type.clone(),
                    metadata.total_number,
                    link.filter,
                    tx.clone(),
                );
                let cloned = processor.clone();
                tasks.push(tokio::spawn(async move {
                    cloned.start_producer().await;
                }));
            }
        }

        futures::future::join_all(tasks).await;
        //merge_mpsc_to_stream(&mut recievers);
    }
}
