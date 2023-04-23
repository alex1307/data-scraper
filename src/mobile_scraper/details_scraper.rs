use std::{
    cmp::max,
    thread,
    time::{Duration, SystemTime},
};

use crossbeam::channel::Sender;
use log::{error, info};

pub const THREADS: usize = 4;

use crate::mobile_scraper::{
    data_processor::{self, DataProcessor},
    get_pages,
    model::{Identity, Message, MobileDetails, MobileList},
    parse_details,
};

pub fn read_list(source_file: &str, links: Vec<String>, sender: &Sender<Message<MobileDetails>>) {
    if links.is_empty() {
        error!("No links to process");
        return;
    }

    let data_processor: DataProcessor<MobileList> =
        data_processor::DataProcessor::from_file(source_file).unwrap();

    let threads = links.len();
    let values = data_processor.get_values();
    let chunk_size = values.len() / threads;
    info!("Chunk size: {}", chunk_size);
    let payloads = values.chunks(chunk_size).collect::<Vec<_>>();

    let mut idx = 0;
    // let mut handles = vec![];
    for arr in payloads {
        let slink = links.get(idx).unwrap();
        let param_slink = slink.clone();
        let params = arr.to_vec();
        let param_sender = sender.clone();
        info!("Sending slink: {} and records {}", slink, params.len());
        // let jh = thread::spawn(move || {
        info!("Task {} started", idx);
        download_details(param_slink, params, &param_sender); // Your asynchronous code here
        info!("Task {} finished", idx);
        // });
        // handles.push(jh);
        idx += 1;
    }
    // for handle in handles {
    //     handle.join().unwrap(); // Wait for the threads to finish
    // }
    sender.send(Message::Stop).unwrap();
    info!("Sent Stop. Processed {} records", values.len());
}

fn download_details(
    slink: String,
    values: Vec<MobileList>,
    sender: &Sender<Message<MobileDetails>>,
) {
    for ml in &values {
        let url = format!(
            "//www.mobile.bg/pcgi/mobile.cgi?act=4&adv={}&slink={}",
            ml.get_id(),
            slink
        );
        info!("Processing url: {}", url);
        let details: MobileDetails = match parse_details(&url) {
            Ok(d) => d,
            Err(e) => {
                error!("Error parsing details: {}", e);
                continue;
            }
        };
        let wait_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            % 3
            + 1; // min 3 sec, max 10 sec
        thread::sleep(Duration::from_secs(wait_time));
        sender.send(Message::Value(details.clone())).unwrap();
        info!("Sent {:?} records", details);
    }
    info!("Done. Processed {} records", values.len());
}
