use std::{
    cmp::max,
    thread,
    time::{Duration, SystemTime},
};

use log::{error, info};
use tokio::sync::mpsc::Sender;

pub const THREADS: usize = 4;

use crate::downloader::{
    data_processor::{self, DataProcessor},
    get_pages,
    model::{Identity, Message, MobileDetails, MobileList},
    parse_details,
};

pub async fn read_list(source_file: &str, links: Vec<String>, sender: &Sender<Message<MobileList>>) {
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
        download_details(param_slink, params, &param_sender).await; // Your asynchronous code here
        info!("Task {} finished", idx);
        // });
        // handles.push(jh);
        idx += 1;
    }
    // for handle in handles {
    //     handle.join().unwrap(); // Wait for the threads to finish
    // }
    sender.send(Message::Stop).await;
    info!("Sent Stop. Processed {} records", values.len());
}



async fn download_details(
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
        sender.send(Message::Value(details.clone())).await;
        info!("Sent {:?} records", details);
    }
    info!("Done. Processed {} records", values.len());
}
