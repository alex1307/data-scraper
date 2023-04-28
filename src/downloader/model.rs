use chrono::Local;
use crossbeam::channel::{self, Receiver};
use log::{error, info};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;

use crate::config::MobileConfig::ConfigData;
use crate::listing_url;
use crate::model::MobileList::MobileList;
use crate::writer::DataPersistance::{create_empty_csv, MobileData, MobileDataWriter};


use super::{get_pages, get_header_data, get_vehicles_prices};
use super::mobile_utils::extract_ascii_latin;
use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Duration;






pub struct Producer<DataMessage>{
    sender: Sender<Message<DataMessage>>,
    config: ConfigData,
}

impl <DataMessage>  Producer<DataMessage> {
    pub fn start(sender: Sender<Message<DataMessage>>, config: ConfigData) -> Self {
        Producer {
            sender,
            config,
        }
    }

}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VehicleList{
    pub results: Vec<MobileList>,
    empty: bool,
}

impl VehicleList {
    pub fn from_url(url:&str, dealer: String) -> Self {
        if let Ok(html) = get_pages(&url){
            let mut vehicle_prices: Vec<MobileList> = get_vehicles_prices(&html);
            for vehicle in vehicle_prices.iter_mut() {
                vehicle.dealer = dealer.clone();
            }
            return VehicleList{
                empty: vehicle_prices.is_empty(),
                results: vehicle_prices,
                
            }
        }
        VehicleList{
            empty: true,
            results: vec![],
        }
    }

    pub fn results(&self) -> Vec<MobileList> {
        self.results.clone()
    }

    pub fn promoted(&self) -> Vec<MobileList> {
        self.results.iter().filter(|x| x.promoted).cloned().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.empty
    }
}

