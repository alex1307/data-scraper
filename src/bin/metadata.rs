use data_scraper::{
    config::{app_config::AppConfig, links::Mobile},
    model::meta::{searches, statistic, MetaHeader},
    services::file_processor::{self, DataProcessor},
    utils::configure_log4rs,
};
use log::info;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_config = AppConfig::from_file("config/config.yml");
    let logger_file_name = format!("{}/meta_log4rs.yml", app_config.get_log4rs_config());
    let metadata_data_file_name = format!("{}/meta_data.csv", app_config.get_data_dir());
    let scraper_config_file = app_config.get_scraper_config();
    let _mobile_config: Mobile = Mobile::from_file(scraper_config_file);
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
