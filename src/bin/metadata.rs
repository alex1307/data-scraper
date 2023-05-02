use data_scraper::{
    config::links::{ConfigData, Mobile},
    downloader::Scraper::{get_header_data, get_pages},
    model::meta::MetaHeader,
    services::FileProcessor::{self, DataProcessor},
    utils::{configure_log4rs, listing_url},
};
use log::info;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    configure_log4rs();
    let mobile_config: Mobile = Mobile::from_file("config/mobile_config.yml");
    info!("Config {:?}", mobile_config);
    let dealer_meta_data = process(mobile_config.config[0].clone());
    let private_meta_data = process(mobile_config.config[1].clone());
    info!("Dealer Meta Data {:?}", dealer_meta_data);
    info!("Private Meta Data {:?}", private_meta_data);
    Ok(())
}

fn process(config: ConfigData) -> Vec<MetaHeader> {
    let mut data = vec![];
    for link_config in config.links.iter() {
        let listing_url = listing_url(&link_config.link, 1);
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
        FileProcessor::DataProcessor::from_file("resources/data/mobile_meta_data.csv");
    meta_data_processor.process(&data, None);
    data
}
