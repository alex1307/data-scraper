use std::{collections::HashMap, thread, time::Duration};

use crossbeam_channel::Sender;
use log::{error, info};

use crate::{downloader::scraper::process_link, utils::details_url};

#[derive(Debug, Clone)]
pub struct DetailsProcessor {
    pub slink: String,
    pub ids: Vec<String>,
    producer: Sender<HashMap<String, String>>,
}

impl DetailsProcessor {
    pub fn new(slink: String, ids: Vec<String>, producer: Sender<HashMap<String, String>>) -> Self {
        DetailsProcessor {
            slink,
            ids,
            producer,
        }
    }

    pub async fn start_producer(self) {
        info!(
            "start producer for slink: {} and ids: {}",
            self.slink,
            &self.ids.len()
        );
        for id in self.ids {
            let url = details_url(&self.slink, &id);
            let response = process_link(&url).await;
            if !response.is_empty() {
                let details: HashMap<String, String> = response[0].clone();
                match self.producer.send(details) {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Error sending data to channel: {}", e);
                    }
                }
            }
            thread::sleep(Duration::from_millis(250));
        }
        info!("producer for slink: {} finished", self.slink);
    }
}
