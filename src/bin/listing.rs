use std::{collections::HashMap, vec};

use data_scraper::{
    config::links::Mobile,
    model::{enums::Payload, list::MobileList, meta::MetaHeader},
    services::streamer::DataStream,
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
                let (tx, rx) = crossbeam_channel::unbounded::<Payload<HashMap<String, String>>>();
                recievers.push(rx);
                let metadata = MetaHeader::from_slink(&link.link);
                let pages: Vec<String> = (1..=metadata.page_numbers())
                    .map(|n| n.to_string())
                    .collect();
                let mut processor = DataStream::new(link.clone(), pages, tx.clone());
                tasks.push(tokio::spawn(async move {
                    processor.stream().await;
                }));
            }
        }

        futures::future::join_all(tasks).await;
        //merge_mpsc_to_stream(&mut recievers);
    }
}
