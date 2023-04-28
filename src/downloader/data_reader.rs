use std::{fmt::Debug, time::Instant};

use log::{info, error};
use serde::{de::DeserializeOwned, Serialize};
use tokio::{process, sync::mpsc::Sender};

use crate::{config::MobileConfig::{ConfigData, LinkData}, listing_url, downloader::model::{MetaHeader, VehicleList}};

use super::model::{Header, Identity, Message};

pub struct DataReader<T: Clone + DeserializeOwned + Serialize + Identity + Header + Debug> {
    sender: Sender<Message<T>>,
    config: ConfigData,
}

impl<T: Clone + DeserializeOwned + Serialize + Identity + Header + Debug> DataReader<T> {
    pub fn start(sender: Sender<Message<T>>, config: ConfigData) -> Self {
        DataReader { sender, config }
    }

    pub async fn run(&self) {
        let dealer = self.config.dealear_type.clone();
        for link in &self.config.links {
            let slink = link.link.clone();
            if slink.is_empty() {
                error!("Empty link found in config");
                continue;
            }
        info!("Reading data from {}", slink);
    }
            
        let start_time = Instant::now();
        let first_page_url = listing_url(&slink, 1);
        let headar_data = MetaHeader::from_url(&first_page_url);
        let pages = headar_data.page_numbers();
        if pages == 0 {
            error!("No pages found for {}", slink);
            return ;
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
            let url = listing_url(&slink, i as i32);
            let results = VehicleList::from_url(&url, dealer.clone());
            if search_promoted_only {
                let promoted = results.promoted();
                if promoted.is_empty() {
                    break;
                }
                sender.send(Message::Data(promoted.clone())).await;
            } else {
                sender.send(value)
            }

            // wait(min_wait_time, max_wait_time);
        }
        let end_time = Instant::now();
        info!(
            "Downloaded {} pages in {} seconds",
            pages,
            end_time.duration_since(start_time).as_secs()
        );
    }
}
