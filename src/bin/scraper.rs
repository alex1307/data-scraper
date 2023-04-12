use std::{
    thread,
    time::{Duration, SystemTime},
};

use chrono::{Local, DateTime};
use data_scraper::{
    mobile_scraper::{
        get_found_result, get_links, get_pages, get_vehicles_prices,
        model::{MetaHeader, SearchRequest, VehiclePrice}, data_processor, parse_details,
    },
    writer::data_persistance::{MobileData, MobileDataWriter}, configure_log4rs,
};

fn main() {
    configure_log4rs();
    //scrape_sold_vehicles("//www.mobile.bg/pcgi/mobile.cgi?act=3&slink=s0ozx4&f1=1");
    scrape_vehicle_details("//www.mobile.bg/pcgi/mobile.cgi?act=4&adv=11680172133317294&slink=s0ozx4")
}

//https://www.mobile.bg/pcgi/mobile.cgi?act=3&slink=s0ozx4&f1=1
fn scrape_sold_vehicles(url: &str) {
    let mut data_processor = data_processor::DataProcessor::from_file("resources/data/mobile_sold.csv").unwrap();
    let html = get_pages(url).unwrap();
    let meta_content = get_found_result(&html).unwrap();
    let meta_data = MetaHeader::from_string(&meta_content);
    let vehicle_prices: Vec<VehiclePrice> = get_vehicles_prices(&html);
    data_processor.process(&vehicle_prices);
    data_processor.save().unwrap();
    if meta_data.total_number > 20 {
        let all_pages = meta_data.total_number / 20;
        let second_page = &get_links(&html).unwrap()[0];
        for page in 2..all_pages + 2 {
            let url = second_page.replace("f1=2", format!("f1={}", page).as_str());
            let page_content = get_pages(url.as_str()).unwrap();
            let page_vehicle = get_vehicles_prices(&page_content);
            data_processor.process(&page_vehicle);
            data_processor.save().unwrap();
            let wait_time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                % 11;
            thread::sleep(Duration::from_secs(wait_time));
        }
    }
}

fn scrape_vehicle_details(adv_url: &str) {
    let html = get_pages(adv_url).unwrap();
    parse_details(&html);
}