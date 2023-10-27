use data_scraper::{
    config::app_config::AppConfig,
    model::search_metadata::{asearches, astatistic, SearchMetadata},
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
    let searches = asearches().await;
    let statistics = astatistic().await;
    all.extend(searches);
    all.extend(statistics);
    for meta in all.iter() {
        info!("{:?}", meta.clone());
    }
    let mut meta_data_processor: DataProcessor<SearchMetadata> =
        file_processor::DataProcessor::from_files(vec![metadata_data_file_name.as_str()]);
    meta_data_processor.process(&all, None);
    Ok(())
}
