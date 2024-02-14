use std::fs;

use log::{error, info};

use crate::kafka::KafkaProducer::{create_producer, send_message};

pub async fn readDataDir(dir: &str, broker: &str, topic: &str) {
    let mut files = Vec::new();
    for result in fs::read_dir(dir).unwrap() {
        match result {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    files.push(path);
                }
            }
            Err(e) => {
                error!("Error: {:?}", e);
                continue;
            }
        }
    }
    let producer = create_producer(broker);

    for file_name in files {
        info!("Reading file: {:?}", file_name);
        match fs::read_to_string(file_name) {
            Ok(content) => {
                send_message(&producer, topic, content.as_bytes().to_vec()).await;
            }
            Err(e) => {
                error!("Error: {:?}", e);
                continue;
            }
        }
    }
}
