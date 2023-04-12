use log::info;
use serde::{Deserialize, Serialize};

use super::currency::Currency;
use super::mobile_utils::extract_ascii_latin;
use std::collections::HashMap;

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

#[derive(Debug, Clone, PartialEq)]
pub struct MetaHeader {
    pub make: String,
    pub model: String,
    pub total_number: u32,
    pub min_price: u32,
    pub max_price: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehiclePrice {
    pub id: String,
    pub make: String,
    pub model: String,
    pub price: u32,
    pub promoted: bool,
    pub sold: bool,
    pub millage: u32,
    pub year: u16,
    pub currency: Currency,
    pub created_on: u64,
    pub updated_on: Option<u64>,
}

impl PartialEq for VehiclePrice {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.price == other.price
            && self.promoted == other.promoted
            && self.sold == other.sold
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
    pub fn from_string(raw: &str) -> Self {
        let meta = extract_ascii_latin(raw);
        let re = regex::Regex::new(r" {2,}").unwrap();
        let split: Vec<&str> = re.split(meta.trim()).collect();
        for s in split.clone() {
            info!("split: {}", s);
        }
        if split.len() <= 4 {
            let min_price = split[0].replace(" ", "").parse::<u32>().unwrap_or(0);
            let max_price = split[1].replace(" ", "").parse::<u32>().unwrap_or(0);
            let total_number = split[2].replace(" ", "").parse::<u32>().unwrap_or(0);
            return MetaHeader {
                make: "".to_string(),
                model: "".to_string(),
                min_price,
                max_price,
                total_number,
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
            make: make.to_string(),
            model: model.to_string(),
            min_price: min,
            max_price: max,
            total_number: total_number,
        }
    }
}

impl VehiclePrice {
    pub fn new(id: String, make: String, model: String, price: u32, currency: Currency) -> Self {
        VehiclePrice {
            id,
            make,
            model,
            price,
            currency,
            sold: false,
            created_on: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            updated_on: None,
            promoted: false,
            millage: 0,
            year: 0,
        }
    }
}
