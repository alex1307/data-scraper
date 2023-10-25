use std::{collections::HashMap, thread, time::Duration};

use crossbeam_channel::Sender;
use log::{error, info};

use crate::{model::enums::Payload, scraper::agent::scrape, utils::mobile_search_url};

#[derive(Debug, Clone)]
pub struct DataStream {
    url: String,
    slink: String,
    dealer: String,
    pub source: Vec<String>,
    producer: Sender<Payload<HashMap<String, String>>>,
    error_handler: Option<Sender<Payload<HashMap<String, String>>>>,
    running: bool,
}

impl DataStream {
    pub fn new(
        url: String,
        slink: String,
        dealer: String,
        source: Vec<String>,
        producer: Sender<Payload<HashMap<String, String>>>,
    ) -> Self {
        DataStream {
            url,
            slink,
            dealer,
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
                crate::model::enums::Dealer::ALL,
                crate::model::enums::SaleType::NONE,
            );
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
            Payload::Data(data) => {
                let mut values = vec![];
                for m in data {
                    let mut dealer: HashMap<String, String> = m;
                    if let Some(value) = dealer.get("promoted") {
                        if "false" == value {
                            self.running = false;
                            break;
                        }
                    }

                    dealer.insert("dealer".to_string(), self.dealer.clone());
                    values.push(dealer);
                }
                Payload::Data(values)
            }
            _ => payload,
        }
    }
}
