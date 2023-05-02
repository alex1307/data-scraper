use std::collections::HashMap;

use crossbeam_channel::Sender;
use log::{error, info};

use crate::{
    downloader::Scraper::process_link,
    utils::{listing_url, wait},
};

#[derive(Debug, Clone)]
pub struct ListProcessor {
    pub slink: String,
    pub name: String,
    pub dealer: String,
    pub pages: u32,
    pub promoted: bool,
    producer: Sender<HashMap<String, String>>,
}

impl ListProcessor {
    pub fn new(
        slink: String,
        name: String,
        dealer: String,
        pages: u32,
        promoted: bool,
        producer: Sender<HashMap<String, String>>,
    ) -> Self {
        ListProcessor {
            slink,
            name,
            dealer,
            pages,
            promoted,
            producer,
        }
    }

    pub async fn start_producer(self) {
        info!("start producer for slink: {}", self.slink);
        for page_number in 1..self.pages {
            let url = listing_url(&self.slink, page_number);
            let results = process_link(&url).await;
            for m in results {
                let mut dealer: HashMap<String, String> = m;
                dealer.insert("slink".to_string(), self.slink.clone());
                dealer.insert("name".to_string(), self.name.clone());
                if self.promoted {
                    if let Some(value) = dealer.get("promoted") {
                        if "false" == value {
                            info!(
                                "link: {}, processed page: {}/{}.",
                                self.slink, page_number, self.pages
                            );
                            info!(
                                "producer for name: {}, dealer: {} and slink: {} finished",
                                self.name, self.dealer, self.slink
                            );
                            return;
                        }
                    }
                }
                dealer.insert("dealer".to_string(), self.dealer.clone());
                match self.producer.send(dealer) {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Error sending data to channel: {}", e);
                    }
                }
            }
            info!(
                "link: {}, processed page: {}/{}.",
                self.slink, page_number, self.pages
            );
            wait(3, 7);
        }
        info!(
            "producer for name: {}, dealer: {} and slink: {} finished",
            self.name, self.dealer, self.slink
        );
    }

}
