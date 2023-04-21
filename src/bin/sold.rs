use std::{time::{SystemTime, Duration}, thread::{self}, vec};

use crossbeam::channel::Sender;
use data_scraper::{
    configure_log4rs,
    mobile_scraper::{
        data_processor::{self, DataProcessor}, get_pages, get_vehicles_prices,
        model::{MobileDetails, Message, Processor, MobileList},
        parse_details,
    },
};
use log::info;


 fn main() {
    configure_log4rs();
    let _html = get_pages("//www.mobile.bg/pcgi/mobile.cgi?act=3&slink=s31et9&f1=1")
        .unwrap();
    // let meta_header = meta_data(&html, "SOLD");
    // info!("Extracted meta data {:?}", meta_header);
    let (tx, rx) = crossbeam::channel::unbounded::<Message<MobileList>>();
    let mut processor = Processor::new(rx, "resources/data/mobile_sold_1.csv");
    let mut tasks = vec![];
    tasks.push(thread::spawn(move || processor.handle()));
    tasks.push(thread::spawn(move || download_vehicle_prices("s3gfw0", "SOLD", "resources/data/mobile_sold_1.csv", &tx.clone(), 1, 27)));
    
    for task in tasks {
        task.join().unwrap();
    }   
    
}

 fn download_vehicle_prices(slink:&str, search_type: &str, source_file: &str,sender: &Sender<Message<MobileList>>, start: i32, end: i32) {
    let mut data_processor =
        data_processor::DataProcessor::from_file(source_file).unwrap();
     
    for i in start..end{
        let url = format!("//www.mobile.bg/pcgi/mobile.cgi?act=3&slink={}&f1={}", slink, i);
        let html = get_pages(url.as_str()).unwrap();
        let vehicle_prices: Vec<MobileList> = get_vehicles_prices(&html);
        let new = data_processor.new_values(&vehicle_prices);
        let wait_time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                % 7 + 3; // min 3 sec, max 10 sec
            thread::sleep(Duration::from_secs(wait_time));
            sender.send(Message::Data(new.clone())).unwrap();
        info!("Sent {} records", new.len());
    }
    sender.send(Message::Stop).unwrap();
    info!("Sent Stop");
}


fn scrape_vehicle_details(vehicles: &Vec<MobileList>) {
    info!("Extracting details for {} vehicles", vehicles.len());
    let mut data_processor: DataProcessor<MobileDetails> =
        data_processor::DataProcessor::from_file("resources/data/details.csv").unwrap();
    let mut details: Vec<MobileDetails> = vec![];
    for vehicle in vehicles {
        info!("Extracting details for {}", vehicle.url);
        let scrapted = parse_details(vehicle.url.as_str()).unwrap();
        details.push(scrapted);
    }
    data_processor.process(&details, None);
}

