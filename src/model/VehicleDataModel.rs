use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::{collections::HashMap, fmt::Debug};

use super::traits::VehicleT;
use super::{
    enums::{Currency, Engine, Gearbox},
    traits::URLResource,
};

#[derive(Debug, Clone)]
pub struct DownloadStatus {
    pub id: String,
    pub source: String,
    pub url: String,
    pub listed: u32,
    pub actual: u32,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Vehicle {
    pub id: String,
    pub source: String,
    pub make: String,
    pub model: String,
    pub title: String,
    pub year: u16,
    pub mileage: u32,
    pub engine: Engine,
    pub gearbox: Gearbox,
    pub power_ps: u32,
    pub power_kw: u32,
    pub currency: Currency,
    pub price: u32,
    pub estimated_price: Option<u32>,
    pub cc: Option<u32>,

    pub url: String,
    pub location: Option<String>,
    pub equipment: Option<String>,
    pub seller_name: Option<String>,
    pub seller_url: Option<String>,

    pub range: Option<u32>,
    pub consumption_fuel: Option<f32>,
    pub consumption_kw: Option<f32>,
    pub co2: Option<u32>,

    pub days_in_sale: Option<u32>,
    pub ranges: Option<String>,
    pub rating: Option<String>,

    #[serde(skip_serializing)]
    pub thresholds: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CarModel {
    //series + relevant models
    pub series: HashMap<String, Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CarMake {
    pub models: HashMap<String, CarModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Eq)]
pub struct LinkId {
    pub url: String,
    pub source: String,
    pub id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Resume {
    pub id: String,
    pub source: String,
    pub title: String,
    pub make: String,
    pub model: String,
    pub modification: String,
    pub currency: Currency,
    pub price: u32,
    pub mileage: u32,
    pub year: u16,
    pub engine: Engine,
    pub gearbox: Gearbox,
    pub promoted: bool,
}

impl URLResource for LinkId {
    fn get_url(&self) -> String {
        self.url.clone()
    }
}

pub trait SearchT {
    fn id(&self) -> String;
    fn source(&self) -> String;
    fn url(&self) -> String;
    fn number_of_cars(&self) -> u32;
    fn actual_number_of_cars(&self) -> u32;
    fn total_pages(&self) -> u32;
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListTrait<T: Clone> {
    pub list: Vec<T>,
}

impl PartialEq for LinkId {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScrapedListData<T: Clone + Serialize + Debug> {
    SingleValue(T),
    Values(Vec<T>),
    Error(String),
}

impl Hash for LinkId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> From<T> for Vehicle
where
    T: VehicleT,
{
    fn from(item: T) -> Self {
        let mut make = item.make();
        if make.starts_with("Mercedes") {
            make = "Mercedes-Benz".to_string();
        } else if make.trim() == "DS" {
            make = "DS Automobiles".to_string();
        } else if make == r#"CITROËN"# {
            make = "Citroen".to_string();
        } else if make.starts_with("Alfa") {
            make = "Alfa Romeo".to_string();
        } else if make.starts_with("SSANG") {
            make = "SsangYong".to_string();
        } else if make.to_uppercase().starts_with("LYNK") {
            make = "Lynk & Co".to_string();
        } else if make.to_uppercase() == "ALPINA" {
            make = "Alpina".to_string();
        }
        Vehicle {
            id: item.id(),
            source: item.source(),
            make,
            model: item.model(),
            title: item.title(),
            currency: item.currency(),
            price: item.price(),
            estimated_price: item.estimated_price(),
            mileage: item.mileage(),
            year: item.year(),
            engine: item.engine(),
            gearbox: item.gearbox(),
            power_ps: item.power_ps(),
            power_kw: item.power_kw(),
            url: item.url(),
            location: item.location(),
            equipment: item.equipment(),
            seller_name: item.seller_name(),
            seller_url: item.seller_url(),
            range: item.range(),
            consumption_fuel: item.consumption_fuel(),
            consumption_kw: item.consumption_kw(),
            co2: item.co2(),
            days_in_sale: item.days_in_sale(),
            cc: item.cc(),
            ranges: item.ranges(),
            rating: item.rating(),
            thresholds: item.thresholds(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::{LOG_CONFIG, utils::helpers::configure_log4rs};

    use super::*;
    use log::info;
    use serde_json;

    #[test]
    fn test_parse_car_makes() {
        configure_log4rs(&LOG_CONFIG);
        let json_data = fs::read_to_string("config/make_and_models.json").unwrap();
        let parsed: Result<HashMap<String, HashMap<String, Vec<String>>>, serde_json::Error> =
            serde_json::from_str(&json_data);
        assert!(parsed.is_ok());
        let car_makes = parsed.unwrap();
        let keys = car_makes.keys();
        assert_eq!(208, keys.len());
        assert!(car_makes.contains_key("Audi"));
        assert!(car_makes.contains_key("BMW"));
        assert!(car_makes.contains_key("Mercedes"));

        info!("Parsed car makes: {:?}", car_makes);
        // Add more assertions as needed to verify the parsed data
    }
}
