use std::{
    thread,
    time::{Duration, Instant, SystemTime},
    vec,
};


use data_scraper::{
    config::MobileConfig::{ConfigData, Mobile, LinkData},
    configure_log4rs, listing_url,
    mobile_scraper::{
        data_processor::{self, DataProcessor},
        model::{MetaHeader, MobileList, VehicleList},
    },
};
use log::{error, info};


fn main() {
    configure_log4rs();
    let mobile_config = Mobile::from_file("config/mobile_config.yml");
    let _ = process(mobile_config.config[0].clone(), "resources/data/dealer.csv");
    let _ = process(mobile_config.config[1].clone(), "resources/data/private.csv");
}

fn process(config: ConfigData, target_file: &str) -> Vec<MobileList> {
    let mut data_processor:DataProcessor<MobileList> = data_processor::DataProcessor::from_file(target_file).unwrap();
    let sold_vehicles: Vec<MobileList> = collect_vehicles(config.sold, config.dealear_type.clone());
    let new_vehicle = collect_vehicles(config.new, config.dealear_type);
    data_processor.process(&new_vehicle, None);
    data_processor.process(&sold_vehicles, None);
    vec![]
}

fn collect_vehicles(link: LinkData, dealer: String) -> Vec<MobileList> {
    let slink = &link.link;
    let start_time = Instant::now();
    let first_page_url = listing_url(slink, 1);
    let headar_data = MetaHeader::from_url(&first_page_url);
    let pages = headar_data.page_numbers();
    if pages == 0 {
        error!("No pages found for {}", slink);
        return vec![];
    }
    let min_wait_time: u64 = 3;
    let max_wait_time: u64 = 10;
    info!(
        "Estimated time to download {} pages should take between {} and {} seconds",
        pages,
        pages * min_wait_time as u32,
        pages * max_wait_time as u32
    );
    let mut mobile_list = vec![];
    let search_promoted_only = &link.name == "NEW";
    info!("Search promoted only {}", search_promoted_only);
    for i in 1..pages + 1 {
        let url = listing_url(slink, i as i32);
        let results = VehicleList::from_url(&url, dealer.clone());
        if search_promoted_only {
            let promoted = results.promoted();
            if promoted.is_empty() {
                break;
            }
            mobile_list.extend(promoted);
        } else {
            mobile_list.extend(results.results());
        }

        wait(min_wait_time, max_wait_time);
    }
    let end_time = Instant::now();
    info!(
        "Downloaded {} pages in {} seconds",
        pages,
        end_time.duration_since(start_time).as_secs()
    );
    mobile_list
}

fn wait(min: u64, max: u64) {
    let wait_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        % max
        + min; // min 3 sec, max 10 sec
    thread::sleep(Duration::from_secs(wait_time));
}