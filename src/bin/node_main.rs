use std::collections::HashMap;

use data_scraper::{
    model::records::MobileRecord,
    services::node::{process_links, save_active_adverts, start_searches},
    utils::{configure_log4rs, create_empty_csv}, CREATED_ON,
};
use log::error;

use data_scraper::CONFIG;

#[tokio::main]
async fn main() {
    let logger_file_name = format!("{}/meta_log4rs.yml", CONFIG.get_log4rs_config());
    configure_log4rs(&logger_file_name);
    let fn_insale = format!("{}/vehicle-{}.csv", CONFIG.get_data_dir(), CREATED_ON.clone());
    let fn_all = format!("{}/vehicle.archive.csv", CONFIG.get_data_dir());

    if let Err(_) = create_empty_csv::<MobileRecord>(&fn_insale) {
        error!("Failed to create file {}", fn_insale);
    }

    if let Err(_) = create_empty_csv::<MobileRecord>(&fn_all) {
        error!("Failed to create file {}", fn_all);
    }
    let (link_producer, mut link_consumer) = crossbeam::channel::unbounded::<String>();
    let (details_producer, mut details_consumer) =
        crossbeam::channel::unbounded::<HashMap<String, String>>();
    let (log_producer, mut log_consumer) = crossbeam::channel::unbounded::<String>();
    let task = tokio::spawn(async move {
        start_searches(link_producer).await;
    });
    let task2 = tokio::spawn(async move {
        process_links(&mut link_consumer, details_producer).await;
    });
    let task3 = tokio::spawn(async move {
        save_active_adverts(&mut details_consumer, log_producer).await;
    });

    tokio::join!(task, task2, task3);
}
