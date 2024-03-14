use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::{collections::HashMap, fmt::Debug};

use super::{
    enums::{Currency, Engine, Gearbox},
    traits::URLResource,
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
    pub equipment: String,
    pub is_dealer: bool,
    pub seller_name: String,
}

impl DetailedVehicleInfo {
    pub fn new(id: String, equipment: String) -> Self {
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
pub trait ConsumptionsT {
    fn get_id(&self) -> String;
    fn source(&self) -> String;
    fn make(&self) -> String;
    fn model(&self) -> String;
    fn year(&self) -> u16;
    fn co2_emission(&self) -> u32;
    fn fuel_consumption(&self) -> Option<f32>;
    fn kw_consuption(&self) -> Option<f32>;
}
pub trait DetailsT {
    fn get_id(&self) -> String;
    fn source(&self) -> String;
    fn phone(&self) -> String;
    fn location(&self) -> String;
    fn view_count(&self) -> u32;
    fn cc(&self) -> u32;
    fn fuel_consumption(&self) -> f64;
    fn electric_drive_range(&self) -> f64;
    fn is_dealer(&self) -> bool;
    fn seller_name(&self) -> String;
    fn equipment(&self) -> String;
}

pub trait ChangeLogT {
    fn get_id(&self) -> String;
    fn source(&self) -> String;
    fn published_on(&self) -> String;
    fn last_modified_on(&self) -> String;
    fn last_modified_message(&self) -> String;
    fn days_in_sale(&self) -> Option<u32>;
    fn sold(&self) -> bool;
    fn promoted(&self) -> bool;
}

pub trait PriceT {
    fn id(&self) -> String;
    fn source(&self) -> String;
    fn estimated_price(&self) -> Option<u32>;
    fn price(&self) -> u32;
    fn currency(&self) -> Currency;
    fn save_difference(&self) -> u32;
    fn overpriced_difference(&self) -> u32;
    fn ranges(&self) -> Option<String>;
    fn rating(&self) -> Option<String>;
    fn thresholds(&self) -> Vec<u32>;
}
pub trait BasicT {
    fn id(&self) -> String;
    fn source(&self) -> String;
    fn make(&self) -> String;
    fn model(&self) -> String;
    fn title(&self) -> String;
    fn currency(&self) -> Currency;
    fn price(&self) -> Option<u32>;
    fn millage(&self) -> Option<u32>;
    fn month(&self) -> Option<u16>;
    fn year(&self) -> u16;
    fn engine(&self) -> Engine;
    fn gearbox(&self) -> Gearbox;
    fn cc(&self) -> u32;
    fn power_ps(&self) -> u32;
    fn power_kw(&self) -> u32;
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

impl<T> From<T> for BaseVehicleInfo
where
    T: BasicT,
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

        BaseVehicleInfo {
            // Assuming `BaseVehicleInfo` has these fields. You need to adjust according to the actual struct fields.
            id: item.id(),
            source: item.source(),
            make,
            model: item.model(),
            title: item.title(),
            currency: item.currency(),
            price: item.price(),
            millage: item.millage(),
            month: item.month(),
            year: item.year(),
            engine: item.engine(),
            gearbox: item.gearbox(),
            cc: item.cc(),
            power_ps: item.power_ps(),
            power_kw: item.power_kw(),
        }
    }
}

impl<T> From<T> for Price
where
    T: PriceT,
{
    fn from(item: T) -> Self {
        Price {
            // Assuming `Price` has these fields. You need to adjust according to the actual struct fields.
            id: item.id(),
            source: item.source(),
            estimated_price: item.estimated_price(),
            price: item.price(),
            currency: item.currency(),
            save_difference: item.save_difference(),
            overpriced_difference: item.overpriced_difference(),
            ranges: item.ranges(),
            rating: item.rating(),
            thresholds: item.thresholds(),
        }
    }
}

impl<T> From<T> for Consumption
where
    T: ConsumptionsT,
{
    fn from(item: T) -> Self {
        Consumption {
            // Assuming `Consumption` has these fields. You need to adjust according to the actual struct fields.
            id: item.get_id(),
            source: item.source(),
            make: item.make(),
            model: item.model(),
            year: item.year(),
            co2_emission: 0,
            fuel_consumption: None,
            kw_consuption: None,
        }
    }
}
impl<T> From<T> for VehicleChangeLogInfo
where
    T: ChangeLogT,
{
    fn from(record: T) -> Self {
        Self {
            id: record.get_id(),
            source: record.source(),
            published_on: record.published_on(),
            last_modified_on: record.last_modified_on(),
            last_modified_message: record.last_modified_message(),
            days_in_sale: record.days_in_sale(),
            sold: record.sold(),
            promoted: record.promoted(),
        }
    }
}

impl<T> From<T> for DetailedVehicleInfo
where
    T: DetailsT,
{
    fn from(record: T) -> Self {
        Self {
            id: record.get_id(),
            source: record.source(),
            phone: record.phone(),
            location: record.location(),
            view_count: record.view_count(),
            cc: record.cc(),
            fuel_consumption: record.fuel_consumption(),
            electric_drive_range: record.electric_drive_range(),
            is_dealer: record.is_dealer(),
            seller_name: record.seller_name(),
            equipment: record.equipment(),
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
