use std::{collections::HashMap, vec};

use data_scraper::{
    config::links::Mobile,
    config_files, configure_log4rs,
    model::{list::MobileList, meta::MetaHeader},
    services::PageProcessor::ListProcessor,
};

use tokio::sync::mpsc;
#[tokio::main]
async fn main() {
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
                let cloned = processor.clone();
                tasks.push(tokio::spawn(async move {
                    cloned.start_producer().await;
                }));
            }
        }

        futures::future::join_all(tasks).await;
    }
}
