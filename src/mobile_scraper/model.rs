use chrono::Local;
use crossbeam::channel::{Receiver, self};
use log::{info, error};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::writer::data_persistance::{create_empty_csv, MobileData, MobileDataWriter};

use super::currency::{Currency, Engine, Gearbox};
use super::mobile_utils::extract_ascii_latin;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub struct SearchRequest {
    pub act: u8,
    pub make: String,
    pub model: String,
    pub engine: String,
    pub transmission: String,
    pub from_year: u16,
    pub to_year: u16,
    pub lpg: bool,
    pub four_wheel_drive: bool,
    pub registration_number: bool,
    pub latest: bool,
    pub sold: bool,
}
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResponse {
    pub slink: String,
    pub links: Vec<String>,
    pub make: String,
    pub model: String,
    pub number_of_vehicle: u16,
    pub min_price: f32,
    pub max_price: f32,
    pub sum_of_prices: f32,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct MetaHeader {
    pub timestamp: String,
    pub meta_type: String,
    pub make: String,
    pub model: String,
    pub total_number: u32,
    pub min_price: u32,
    pub max_price: u32,
    pub created_on: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MobileList {
    pub id: String,
    pub make: String,
    pub model: String,
    pub currency: Currency,
    pub price: u32,
    pub millage: u32,
    pub year: u16,
    pub promoted: bool,
    pub sold: bool,
    pub url: String,
    pub created_on: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MobileDetails {
    pub id: String,
    #[serde(skip)]
    pub engine: Engine,
    #[serde(skip)]
    pub gearbox: Gearbox,

    pub currency: Currency,
    pub price: u32,
    pub power: u16,
    pub phone: String,
    pub view_count: u32,
    #[serde(skip)]
    pub extras: Vec<String>,
    pub equipment: u64,
    pub created_on: String,
}

impl PartialEq for MobileList {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.price == other.price
            && self.promoted == other.promoted
            && self.sold == other.sold
    }
}

impl MobileDetails {
    pub fn new(id: String, phone: String) -> Self {
        MobileDetails {
            id,
            phone,
            created_on: Local::now().format("%Y-%m-%d").to_string(),
            ..Default::default()
        }
    }
}

impl SearchRequest {
    pub fn new(make: String, model: String) -> Self {
        SearchRequest {
            make,
            model,
            engine: String::new(),
            transmission: String::new(),
            from_year: 0,
            to_year: 0,
            lpg: false,
            four_wheel_drive: false,
            registration_number: false,
            act: 3,
            latest: false,
            sold: false,
        }
    }

    pub fn set_engine(&mut self, engine: String) {
        self.engine = engine;
    }

    pub fn set_transmission(&mut self, transmission: String) {
        self.transmission = transmission;
    }

    pub fn set_from_year(&mut self, from_year: u16) {
        self.from_year = from_year;
    }

    pub fn set_to_year(&mut self, to_year: u16) {
        self.to_year = to_year;
    }

    pub fn set_sold(&mut self, sold: bool) {
        self.sold = sold;
    }

    pub fn set_latest(&mut self, latest: bool) {
        self.latest = latest;
    }

    pub fn set_act(&mut self, act: u8) {
        self.act = act;
    }

    pub fn to_form_data(&self) -> HashMap<String, String> {
        let mut form_data = HashMap::new();
        form_data.insert("rub_pub_save".to_string(), 1.to_string());
        form_data.insert("rub".to_string(), 1.to_string());
        form_data.insert("act".to_string(), self.act.to_string());

        form_data.insert("f1".to_string(), 1.to_string());
        form_data.insert("f2".to_string(), 1.to_string());
        form_data.insert("f3".to_string(), 1.to_string());
        form_data.insert("f4".to_string(), 1.to_string());
        form_data.insert("f9".to_string(), "лв.".to_string());
        form_data.insert("f21".to_string(), "01".to_string());
        for i in 39..132 {
            let key = format!("f{}", i).to_string();
            form_data.insert(key.clone(), 0.to_string());
        }
        if !self.make.is_empty() {
            form_data.insert("f5".to_string(), self.make.clone());
        }

        if !self.model.is_empty() {
            form_data.insert("f6".to_string(), self.model.clone());
        }

        if self.from_year > 1950 {
            form_data.insert("f10".to_string(), self.from_year.to_string());
        }

        if self.to_year > 1950 {
            form_data.insert("f11".to_string(), self.to_year.to_string());
        }

        if !self.engine.is_empty() {
            form_data.insert("f12".to_string(), self.engine.clone());
        }

        if !self.transmission.is_empty() {
            form_data.insert("f13".to_string(), self.transmission.clone());
        }

        if self.four_wheel_drive {
            form_data.insert("88".to_string(), 1.to_string());
        }

        if self.lpg {
            form_data.insert("92".to_string(), 1.to_string());
        }

        if self.registration_number {
            form_data.insert("102".to_string(), 1.to_string());
        }

        if self.latest {
            form_data.insert("f20".to_string(), 7.to_string());
        } else {
            form_data.insert("f20".to_string(), 1.to_string());
        }

        if self.sold {
            form_data.insert("f94".to_string(), "1".to_string());
        }
        info!("form_data: {:?}", &form_data);

        return form_data;
    }
}


impl MetaHeader {
    pub fn from_string(raw: &str, meta_type: String) -> Self {
        let meta = extract_ascii_latin(raw);
        let re = regex::Regex::new(r" {2,}").unwrap();
        let split: Vec<&str> = re.split(meta.trim()).collect();
        for s in split.clone() {
            info!("split: {}", s);
        }
        let timestamp = chrono::Utc::now().timestamp().to_string();
        if split.len() <= 4 {
            let min_price = split[0].replace(" ", "").parse::<u32>().unwrap_or(0);
            let max_price = split[1].replace(" ", "").parse::<u32>().unwrap_or(0);
            let total_number = split[2].replace(" ", "").parse::<u32>().unwrap_or(0);
            return MetaHeader {
                timestamp,
                meta_type,
                make: "ALL".to_string(),
                model: "ALL".to_string(),
                min_price,
                max_price,
                total_number,
                created_on: chrono::Local::now().format("%Y-%m-%d").to_string(),
            };
        }

        let make_model: Vec<&str> = split[0].split_whitespace().collect();

        let (make, model) = if make_model.len() == 1 {
            (make_model[0], "")
        } else {
            (make_model[0], make_model[1])
        };

        let min = split[1].replace(" ", "").parse::<u32>().unwrap_or(0);
        let max = split[2].replace(" ", "").parse::<u32>().unwrap_or(0);
        let total_number = split[3].replace(" ", "").parse::<u32>().unwrap_or(0);

        MetaHeader {
            timestamp,
            meta_type,
            make: make.to_string(),
            model: model.to_string(),
            min_price: min,
            max_price: max,
            total_number,
            created_on: chrono::Local::now().format("%Y-%m-%d").to_string(),
        }
    }
}

pub trait Identity {
    fn get_id(&self) -> String;
}

impl Identity for MetaHeader {
    fn get_id(&self) -> String {
        self.timestamp.clone()
    }
}

impl Header for MetaHeader {
    fn heder() -> Vec<&'static str> {
        return vec!["timestamp", 
            "meta_type", 
            "make", 
            "model", 
            "total_number", 
            "min_price", 
            "max_price",
            "created_on"];
    }
}

impl MobileList {
    pub fn new(
        id: String,
        make: String,
        model: String,
        price: u32,
        currency: Currency,
        created_on: String,
    ) -> Self {
        MobileList {
            id,
            make,
            model,
            price,
            currency,
            sold: false,
            created_on,
            promoted: false,
            millage: 0,
            url: "".to_string(),
            year: 0,
        }
    }
}

impl Identity for MobileList {
    fn get_id(&self) -> String {
        self.id.clone()
    }
}

impl Identity for MobileDetails {
    fn get_id(&self) -> String {
        self.id.clone()
    }
}

pub trait Header {
    fn heder() -> Vec<&'static str>;
}

impl Header for MobileList {
    fn heder() -> Vec<&'static str> {
        return vec![
            "id",
            "make",
            "model",
            "currency",
            "price",
            "millage",
            "year",
            "promoted",
            "sold",
            "created_on",
        ];
    }
}

impl Header for MobileDetails  {
    fn heder() -> Vec<&'static str> {
        return vec![
            "id",
            "engine",
            "gearbox",
            "power",
            "phone",
            "view_count",
            "equipment",
            "created_on",
        ];
    }
}


pub enum Message<T: Clone + DeserializeOwned + Serialize + Identity + Header> {
    Data(Vec<T>),
    Value(T),
    Stop,   
}

pub struct Processor <T: Clone + DeserializeOwned + Serialize + Identity + Header>{
    receiver: Receiver<Message<T>>,
    data: Vec<T>,
    file: String,
    cache_size: u32,
}

impl <T: Clone + DeserializeOwned + Serialize + Identity + Header> Processor<T>{
    pub fn new(receiver: Receiver<Message<T>>, file: &str) -> Self{
        match create_empty_csv::<T>(file) {
            Ok(_) => info!("File {} created", file),
            Err(e) => error!("Error creating file {}: {}", file, e),
        }
        Self{
            receiver,
            data: vec![],
            file: file.to_string(),
            cache_size: 100,
        }
    }

    pub fn handle(&mut self) {
        info!("Start handling messages");
        loop{
            let mut do_exit = false;
            match self.receiver.recv_timeout(Duration::from_secs(3)) {
                Ok(message) => {
                    match message {
                        Message::Data(data) => {
                            info!("Data message received: {}", data.len());
                            self.data.extend(data);
                        }

                        Message::Stop => {
                            info!("Stop message received");
                            do_exit = true;
                        }
                        Message::Value(value) => {
                            self.data.push(value);
                        },
                    }
                }
                Err(e) => match e {
                    channel::RecvTimeoutError::Timeout => {
                        // No message received within the timeout
                    }
                    channel::RecvTimeoutError::Disconnected => {
                        error!("Channel is disconnected");
                        break;
                    }
                },
            }
            
            // Check if there are more messages available without blocking
            for message in self.receiver.try_iter() {
                match message {
                    Message::Data(data) => {
                        self.data.extend(data);
                    }
                    Message::Stop => {
                        do_exit = true;
                    }
                    Message::Value(_) => todo!(),
                }
            }
            
            if self.data.len() >= self.cache_size as usize {
                let data = MobileData::Payload(self.data.clone());
                data.write_csv(&self.file, false).unwrap();
                info!("write {} records to {}", self.data.len(), &self.file);
                self.data.clear();
            }

            if do_exit {
                break;
            }
        }
        if self.data.len() > 0{
            let data = MobileData::Payload(self.data.clone());
            data.write_csv(&self.file, false).unwrap();
            info!("write {} records to {}", self.data.len(), &self.file);
            self.data.clear();
        }
        
    }
}
