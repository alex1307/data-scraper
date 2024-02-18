use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::{collections::HashMap, fmt::Debug};

use super::records::MobileRecord;
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
    pub month: Option<u16>,
    pub year: u16,
    pub engine: Engine,
    pub gearbox: Gearbox,
    pub cc: u32,
    pub power_ps: u32,
    pub power_kw: u32,
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
    pub ranges: Option<String>,
    pub rating: Option<String>,

    #[serde(skip_serializing)]
    pub thresholds: Vec<u32>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Default)]
pub struct Consumption {
    pub id: String,
    pub source: String,
    pub make: String,
    pub model: String,
    pub year: u16,
    pub co2_emission: u32,
    pub fuel_consumption: Option<f32>,
    pub kw_consuption: Option<f32>,
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

impl Consumption {
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
            "rating",
        ]
    }
}

impl Header for Consumption {
    fn header() -> Vec<&'static str> {
        vec![
            "id",
            "source",
            "make",
            "model",
            "year",
            "co2_emission",
            "fuel_consumption",
            "kw_consuption",
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

impl From<MobileRecord> for BaseVehicleInfo {
    fn from(record: MobileRecord) -> Self {
        Self {
            id: record.id,
            source: record.source,
            title: record.title,
            make: record.make,
            model: record.model,
            currency: record.currency,
            price: Some(record.price),
            millage: Some(record.mileage),
            year: record.year,
            engine: record.engine,
            gearbox: record.gearbox,
            cc: 0,
            power_ps: record.power,
            power_kw: record.power,
            month: None,
        }
    }
}

impl From<MobileRecord> for DetailedVehicleInfo {
    fn from(record: MobileRecord) -> Self {
        Self {
            id: record.id,
            source: record.source,
            phone: record.phone,
            seller_name: record.name,
            location: record.location,
            view_count: record.view_count,
            cc: 0,
            fuel_consumption: 0.0,
            electric_drive_range: 0.0,
            equipment: record.equipment,
            is_dealer: record.dealer,
        }
    }
}

impl From<MobileRecord> for VehicleChangeLogInfo {
    fn from(record: MobileRecord) -> Self {
        Self {
            id: record.id,
            source: record.source,
            published_on: record.created_on,
            last_modified_on: record.updated_on,
            last_modified_message: "".to_string(),
            days_in_sale: None,
            sold: record.sold,
            promoted: record.vip,
        }
    }
}

impl From<MobileRecord> for Price {
    fn from(value: MobileRecord) -> Self {
        Self {
            id: value.id,
            source: value.source,
            estimated_price: None,
            price: value.price,
            currency: value.currency,
            save_difference: 0,
            overpriced_difference: 0,
            ranges: None,
            rating: None,
            thresholds: vec![],
        }
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
