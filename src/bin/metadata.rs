use data_scraper::{
    config::app_config::AppConfig,
    model::meta::{asearch, searches, statistic, MetaHeader},
    services::file_processor::{self, DataProcessor},
    utils::configure_log4rs,
    SEARCH_ALL,
};
use log::info;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_config = AppConfig::from_file("config/config.yml");
    let logger_file_name = format!("{}/meta_log4rs.yml", app_config.get_log4rs_config());
    configure_log4rs(&logger_file_name);
    let all = SEARCH_ALL.clone();
    let meta = asearch(
        data_scraper::model::enums::Dealer::ALL,
        data_scraper::model::enums::SaleType::INSALE,
    )
    .await;
    info!("{:?}", meta);
    info!("----------------------------------------");
    info!("Starting METADATA application");
    info!("Search metadata ALL: {:?}", all);
    info!("----------------------------------------");
    Ok(())
}

async fn read_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let app_config = AppConfig::from_file("config/config.yml");
    let logger_file_name = format!("{}/meta_log4rs.yml", app_config.get_log4rs_config());
    let metadata_data_file_name = format!("{}/meta_data.csv", app_config.get_data_dir());
    configure_log4rs(&logger_file_name);
    let mut all = vec![];
    let searches = searches();
    let statistics = statistic();
    all.extend(searches);
    all.extend(statistics);
    for meta in all.iter() {
        info!("{:?}", meta);
    }
    let mut meta_data_processor: DataProcessor<MetaHeader> =
        file_processor::DataProcessor::from_files(vec![metadata_data_file_name.as_str()]);
    meta_data_processor.process(&all, None);
    Ok(())
}
