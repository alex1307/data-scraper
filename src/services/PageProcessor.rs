use std::collections::HashMap;

use log::info;
use tokio::sync::mpsc::Sender;

use crate::{downloader::Scraper::process_link, listing_url, wait};

#[derive(Debug, Clone)]
pub struct ListProcessor {
    pub slink: String,
    pub dealer: String,
    pub pages: u32,
    pub promoted: bool,
    producer: Sender<HashMap<String, String>>,
}

impl ListProcessor {
    pub fn new(
        slink: String,
        dealer: String,
        pages: u32,
        promoted: bool,
        producer: Sender<HashMap<String, String>>,
    ) -> Self {
        ListProcessor {
            slink,
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
            info!("url: {}", url);
            let results = process_link(&url).await;

            for m in results {
                let mut dealer: HashMap<String, String> = HashMap::from(m);
                if self.promoted && dealer.contains_key("promoted") {
                    let value = dealer.get("promoted").unwrap();
                    if "false" == value {
                        return;
                    }
                }
                dealer.insert("dealer".to_string(), self.dealer.clone());
                self.producer.send(dealer).await.unwrap();
                wait(3, 7);
            }
        }
    }
}
