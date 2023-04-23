use data_scraper::{
    config::MobileConfig::{ConfigData, Mobile},
    configure_log4rs, listing_url,
    mobile_scraper::{
        data_processor::{self, DataProcessor},
        get_header_data, get_pages,
        model::MetaHeader,
    },
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
    let all_listing_url = listing_url(&config.all.link, 1);
    let sold_listing_url = listing_url(&config.sold.link, 1);
    let new_listing_url = listing_url(&config.new.link, 1);

    let mut meta_data_processor: DataProcessor<MetaHeader> =
        data_processor::DataProcessor::from_file("resources/data/mobile_meta_data.csv").unwrap();

    let new_meta_data = create_meta_data(
        &new_listing_url,
        config.new.name,
        config.dealear_type.clone(),
    );
    let sold_meta_data = create_meta_data(
        &sold_listing_url,
        config.sold.name,
        config.dealear_type.clone(),
    );
    let all_meta_data = create_meta_data(
        &all_listing_url,
        config.all.name,
        config.dealear_type.clone(),
    );
    let meta_data = vec![
        all_meta_data.clone(),
        new_meta_data.clone(),
        sold_meta_data.clone(),
    ];
    meta_data_processor.process(&vec![all_meta_data, new_meta_data, sold_meta_data], None);
    meta_data
}

fn create_meta_data(url: &str, meta_type: String, dealer: String) -> MetaHeader {
    let html = get_pages(url).unwrap();
    let meta_content = get_header_data(&html).unwrap();
    let meta_data = MetaHeader::from_string(&meta_content, meta_type, dealer);
    meta_data
}
