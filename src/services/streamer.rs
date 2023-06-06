use std::{collections::HashMap, thread, time::Duration};

use crossbeam_channel::Sender;
use log::{error, info};

use crate::{
    config::links::LinkData,
    downloader::scraper::scrape,
    model::enums::Payload,
    utils::{details_url, listing_url},
};

#[derive(Debug, Clone)]
pub struct DataStream {
    pub config: LinkData,
    pub source: Vec<String>,
    producer: Sender<Payload<HashMap<String, String>>>,
    error_handler: Option<Sender<Payload<HashMap<String, String>>>>,
    running: bool,
}

impl DataStream {
    pub fn new(
        config: LinkData,
        source: Vec<String>,
        producer: Sender<Payload<HashMap<String, String>>>,
    ) -> Self {
        DataStream {
            config,
            source,
            producer,
            error_handler: None,
            running: true,
        }
    }

    pub fn with_error_handler(
        &mut self,
        error_handler: Sender<Payload<HashMap<String, String>>>,
    ) -> &Self {
        self.error_handler = Some(error_handler);
        self
    }

    pub async fn stream(&mut self) {
        info!(
            "start stream for config: {:#?} and \n source.len: {}",
            self.config,
            &self.source.len()
        );
        for value in self.source.clone() {
            let url = if self.config.link_type == "details" {
                details_url(&self.config.link, &value)
            } else {
                listing_url(&self.config.link, &value)
            };
            let mut payload = scrape(&url).await;
            if let Payload::Error(_) = payload {
                if let Some(handler) = &self.error_handler {
                    if let Err(e) = handler.send(payload) {
                        error!("Error: config: {:#?}, error: {}", self.config, e);
                    }
                }
                continue;
            } else {
                payload = self.process_payload(payload);
                if let Err(e) = self.producer.send(payload) {
                    error!(
                        "Error: config: {:#?}, value: {}, error: {}",
                        self.config, value, e
                    );
                }
                if !&self.running {
                    break;
                }
            }

            thread::sleep(Duration::from_millis(750));
        }
        if let Err(e) = self.producer.send(Payload::Done) {
            error!("Error: config: {:#?}, error: {}", self.config, e);
        }

        if let Some(handler) = &self.error_handler {
            if let Err(e) = handler.send(Payload::Done) {
                error!("Error: config: {:#?}, error: {}", self.config, e);
            }
        }

        info!(
            "producer for config: {:#?} finished. Successfully processed: {} items",
            self.config,
            &self.source.len()
        );
    }
    fn process_payload(
        &mut self,
        payload: Payload<HashMap<String, String>>,
    ) -> Payload<HashMap<String, String>> {
        match payload {
            Payload::Data(data) => {
                let mut values = vec![];
                for m in data {
                    let mut dealer: HashMap<String, String> = m;
                    if self.config.filter {
                        if let Some(value) = dealer.get("promoted") {
                            if "false" == value {
                                self.running = false;
                                break;
                            }
                        }
                    }
                    dealer.insert("dealer".to_string(), self.config.dealer.clone());
                    values.push(dealer);
                }
                Payload::Data(values)
            }
            _ => payload,
        }
    }
}
