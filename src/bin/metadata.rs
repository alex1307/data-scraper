use std::{thread::sleep, time::Duration};

use data_scraper::{
    config::app_config::AppConfig,
    model::search_metadata::{searches, statistic, SearchMetadata},
    services::file_processor::{self, DataProcessor},
    utils::configure_log4rs,
};
use log::info;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    read_metadata().await
}

async fn read_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let app_config = AppConfig::from_file("config/config.yml");
    let logger_file_name = format!("{}/meta_log4rs.yml", app_config.get_log4rs_config());
    let metadata_data_file_name = format!("{}/meta_data.csv", app_config.get_data_dir());
    configure_log4rs(&logger_file_name);
    let mut all = vec![];
    let searches = searches();
    //slee for 10 seconds
    info!("searches len: {}", searches.len());
    let statistics = statistic();
    info!("statistics len: {}", statistics.len());
    all.extend(searches);
    all.extend(statistics);
    info!("metadata len: {}", all.len());
    for meta in all.iter() {
        info!("{:?}", meta.clone());
    }
    info!("Metadata file: {}", metadata_data_file_name);
    let mut meta_data_processor: DataProcessor<SearchMetadata> =
        file_processor::DataProcessor::from_files(vec![metadata_data_file_name.as_str()]);
    meta_data_processor.process(&all, None);
    Ok(())
}
