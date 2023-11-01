use std::{collections::HashMap, thread, time::Duration};

use crossbeam_channel::Sender;
use log::{error, info};
use serde_json::from_str;

use crate::{model::enums::Payload, scraper::agent::scrape, utils::mobile_search_url, LISTING_URL};

#[derive(Debug, Clone)]
pub struct DataStream {
    url: String,
    slink: String,
    pub source: Vec<String>,
    producer: Sender<Payload<HashMap<String, String>>>,
    error_handler: Option<Sender<Payload<HashMap<String, String>>>>,
    running: bool,
}

impl DataStream {
    pub fn new(
        url: String,
        slink: String,

        source: Vec<String>,
        producer: Sender<Payload<HashMap<String, String>>>,
    ) -> Self {
        DataStream {
            url,
            slink,
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
        info!("Starting producer for config: {:#?}", self.slink);
        for value in self.source.clone() {
            let url = mobile_search_url(
                &self.url,
                &value,
                &self.slink,
                crate::model::enums::SaleType::NONE,
                0,
                0,
            );
            info!("--> Processing url: {}", url);
            let mut payload = scrape(&url).await;
            if let Payload::Error(_) = payload {
                if let Some(handler) = &self.error_handler {
                    if let Err(e) = handler.send(payload.clone()) {
                        error!("Error: config: {:#?}, error: {}", url, e);
                    } else {
                        info!(
                            "Sent not found error for url: {}. Payload: {:?}",
                            url, payload
                        );
                    }
                }
                continue;
            } else {
                payload = self.process_payload(payload);
                if let Err(e) = self.producer.send(payload) {
                    error!(
                        "Error: config: {:#?}, value: {}, error: {}",
                        self.url, value, e
                    );
                }
                if !&self.running {
                    break;
                }

                if self.url == LISTING_URL {
                    let page_number = from_str::<i32>(&value).unwrap();
                    if page_number > 99 {
                        self.running = false;
                    }
                }
            }
            thread::sleep(Duration::from_millis(750));
        }
        if let Err(e) = self.producer.send(Payload::Done) {
            error!("Error: config: {:#?}, error: {}", self.slink, e);
        }

        if let Some(handler) = &self.error_handler {
            if let Err(e) = handler.send(Payload::Done) {
                error!("Error: config: {:#?}, error: {}", self.slink, e);
            }
        }

        info!(
            "producer for config: {:#?} finished. Successfully processed: {} items",
            self.slink,
            &self.source.len()
        );
    }
    fn process_payload(
        &mut self,
        payload: Payload<HashMap<String, String>>,
    ) -> Payload<HashMap<String, String>> {
        match payload {
            Payload::Data(data) => Payload::Data(data),
            _ => payload,
        }
    }
}
