use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::{collections::HashMap, fmt::Debug};

use super::{
    enums::{Currency, Engine, Gearbox},
    traits::{Header, Identity, URLResource},
};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BaseVehicleInfo {
    pub id: String,
    pub source: String,
    pub make: String,
    pub model: String,
    pub title: String,
    pub currency: Currency,
    pub price: Option<u32>,
    pub millage: Option<u32>,
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
    pub source: String,
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
    pub source: String,
    pub published_on: String,
    pub last_modified_on: String,
    pub last_modified_message: String,
    pub days_in_sale: Option<u32>,
    pub sold: bool,
    pub promoted: bool,
}

impl VehicleChangeLogInfo {
    pub fn new(id: String, source: String) -> Self {
        Self {
            id,
            source,
            ..Default::default()
        }
    }
}
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Default)]
pub struct Price {
    pub id: String,
    pub source: String,
    pub estimated_price: Option<u32>,
    pub price: u32,
    pub currency: Currency,
    pub save_difference: u32,
    pub overpriced_difference: u32,
    pub ranges: Vec<u32>,
}

impl Price {
    pub fn new(id: String, source: String) -> Self {
        Self {
            id,
            source,
            ..Default::default()
        }
    }
}

impl Header for Price {
    fn header() -> Vec<&'static str> {
        vec![
            "id",
            "source",
            "estimated_price",
            "price",
            "currency",
            "save_difference",
            "overpriced_difference",
            "ranges",
        ]
    }
}

impl Header for BaseVehicleInfo {
    fn header() -> Vec<&'static str> {
        vec![
            "id", "source", "make", "model", "title", "currency", "price", "millage", "year",
            "engine", "gearbox", "power",
        ]
    }
}

impl Header for DetailedVehicleInfo {
    fn header() -> Vec<&'static str> {
        vec![
            "id",
            "source",
            "phone",
            "location",
            "view_count",
            "cc",
            "fuel_consumption",
            "electric_drive_range",
            "equipment",
            "is_dealer",
            "seller_name",
        ]
    }
}

impl Header for VehicleChangeLogInfo {
    fn header() -> Vec<&'static str> {
        vec![
            "id",
            "source",
            "published_on",
            "last_modified_on",
            "last_modified_message",
            "days_in_sale",
            "sold",
            "promoted",
        ]
    }
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScrapedListData<T: Identity + Clone + Serialize + Debug> {
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
