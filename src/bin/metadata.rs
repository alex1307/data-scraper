use std::{thread::sleep, time::Duration};

use data_scraper::{
    configure_log4rs,
    mobile_scraper::{
        data_processor::{self, DataProcessor},
        get_found_result, get_pages,
        model::MetaHeader,
    },
};
use log::info;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    configure_log4rs();
    let all_url = "//www.mobile.bg/pcgi/mobile.cgi?act=3&slink=s3gfi1&f1=1";
    let sold_url = "//www.mobile.bg/pcgi/mobile.cgi?act=3&slink=s3gfw0&f1=1";
    let new_sales_url = "//www.mobile.bg/pcgi/mobile.cgi?act=3&slink=s3gfo4&f1=1";
    let all = meta_data(all_url, "ALL");
    info!("ALL data {:?}", all);
    sleep(Duration::from_secs(1));
    let sold = meta_data(sold_url, "SOLD");
    info!("SOLD meta data {:?}", sold);
    sleep(Duration::from_secs(1));
    let sales = meta_data(new_sales_url, "SALE"); 
    info!("SALE data {:?}", sales);
    Ok(())
}

fn meta_data(url: &str, meta_type: &str) -> MetaHeader {
    let html = get_pages(url).unwrap();
    let mut meta_data_processor: DataProcessor<MetaHeader> =
        data_processor::DataProcessor::from_file("resources/data/mobile_meta_data.csv").unwrap();
    let meta_content = get_found_result(&html).unwrap();
    let meta_data = MetaHeader::from_string(&meta_content, meta_type.to_string());
    meta_data_processor.process(&vec![meta_data.clone()], None);
    meta_data
}
