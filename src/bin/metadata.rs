use data_scraper::{
    config::{
        app_config::AppConfig,
        links::{ConfigData, Mobile},
    },
    downloader::scraper::{get_header_data, get_pages},
    model::meta::MetaHeader,
    services::file_processor::{self, DataProcessor},
    utils::{config_files, configure_log4rs, listing_url},
    DATE_FORMAT,
};
use log::info;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_config = AppConfig::from_file("config/config.yml");
    let logger_file_name = format!("{}/meta_log4rs.yml", app_config.get_log4rs_config());
    let metadata_data_file_name = format!("{}/mobile_meta_data.csv", app_config.get_data_dir());
    let scrpaer_config_file = app_config.get_scraper_config();
    let created_on = chrono::Utc::now().format(DATE_FORMAT).to_string();

    configure_log4rs(&logger_file_name);
    info!("----------------------------------------");
    info!("Starting *METADATA* application on {}", created_on);
    info!("scraper config file: {}", scrpaer_config_file);
    info!("listing data file: {}", metadata_data_file_name);
    info!("number of threads: {}", app_config.get_num_threads());
    info!("----------------------------------------");
    let mobile_config: Mobile = Mobile::from_file(scrpaer_config_file);
    info!("Config {:?}", mobile_config);
    config_files::<MetaHeader>(&mobile_config.config);
    let dealer_meta_data = process(mobile_config.config[0].clone(), &metadata_data_file_name);
    let private_meta_data = process(mobile_config.config[1].clone(), &metadata_data_file_name);
    info!("Dealer Meta Data {:?}", dealer_meta_data);
    info!("Private Meta Data {:?}", private_meta_data);
    Ok(())
}

fn process(config: ConfigData, file_name: &str) -> Vec<MetaHeader> {
    let mut data = vec![];
    for link_config in config.links.iter() {
        let listing_url = listing_url(&link_config.link, "1");
        let html = get_pages(&listing_url).unwrap();
        let meta_content = get_header_data(&html).unwrap();
        let meta_data = MetaHeader::from_string(
            &meta_content,
            link_config.name.clone(),
            config.dealear_type.clone(),
        );
        data.push(meta_data);
    }

    let mut meta_data_processor: DataProcessor<MetaHeader> =
        file_processor::DataProcessor::from_file(file_name);
    meta_data_processor.process(&data, None);
    data
}
