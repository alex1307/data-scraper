use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;

use super::{
    enums::{Currency, Engine, Gearbox},
    traits::{Identity, URLResource},
};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BaseVehicleInfo {
    pub id: String,
    pub make: String,
    pub model: String,
    pub title: String,
    pub currency: Currency,
    pub price: u32,
    pub millage: u32,
    pub year: u16,
    pub engine: Engine,
    pub gearbox: Gearbox,
    pub power: u16,
}

impl BaseVehicleInfo {
    pub fn new(id: String) -> Self {
        Self {
            id,
            currency: Currency::EUR,
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DetailedVehicleInfo {
    pub id: String,
    pub phone: String,
    pub location: String,
    pub view_count: u32,
    pub cc: u32,
    pub fuel_consumption: f64,
    pub electric_drive_range: f64,
    pub equipment: u64,
    pub is_dealer: bool,
    pub seller_name: String,
}

impl DetailedVehicleInfo {
    pub fn new(id: String, equipment: u64) -> Self {
        Self {
            id,
            equipment,
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct VehicleChangeLogInfo {
    pub id: String,
    pub published_on: String,
    pub last_modified_on: String,
    pub last_modified_message: String,
    pub days_in_sale: u32,
    pub sold: bool,
    pub promoted: bool,
}

impl VehicleChangeLogInfo {
    pub fn new(id: String, last_modified_message: String) -> Self {
        Self {
            id,
            last_modified_message,
            ..Default::default()
        }
    }
}

pub struct PriceCalculator {
    pub source: String,
    pub make: String,
    pub model: String,
    pub year: u16,
    pub millage: u32,
    pub engine: Engine,
    pub gearbox: Gearbox,
    pub equipment: Option<u64>,
    pub estimated_price: Option<u32>,
    pub ranges: Vec<u32>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Default)]
pub struct LinkId {
    pub url: String,
    pub id: String,
}

impl Identity for LinkId {
    fn get_id(&self) -> String {
        self.id.clone()
    }
}

impl URLResource for LinkId {
    fn get_url(&self) -> String {
        self.url.clone()
    }
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

pub enum ScrapedListData<T: Identity + Clone + Serialize> {
    SingleValue(T),
    Values(Vec<T>),
    Error(String),
}

impl Hash for LinkId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::{utils::helpers::configure_log4rs, LOG_CONFIG};

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
