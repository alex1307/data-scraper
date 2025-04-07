use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::{collections::HashMap, fmt::Debug};

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

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BaseVehicleInfo {
    pub id: String,
    pub source: String,
    pub make: String,
    pub model: String,
    pub title: String,
    pub currency: Currency,
    pub price: Option<u32>,
    pub mileage: Option<u32>,
    pub month: Option<u16>,
    pub year: u16,
    pub engine: Engine,
    pub gearbox: Gearbox,
    pub cc: u32,
    pub power_ps: u32,
    pub power_kw: u32,
    pub search_id: String,
    pub url: String,
}

impl BaseVehicleInfo {
    pub fn new(id: String, source: String) -> Self {
        Self {
            id,
            source,
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DetailedVehicleInfo {
    pub id: String,
    pub source: String,
    pub location: String,
    pub equipment: String,
    pub seller_name: String,
    pub seller_url: String,
    pub range: u32,
    pub consumption_fuel: f32,
    pub consumption_kw: f32,
    pub co2: u32,
    pub days_in_sale: Option<u32>,
}

impl DetailedVehicleInfo {
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

impl Price {
    pub fn new(id: String, source: String) -> Self {
        Self {
            id,
            source,
            ..Default::default()
        }
    }
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
    pub seller_name: String,
    pub seller_url: String,

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

pub trait DetailsT {
    fn get_id(&self) -> String;
    fn source(&self) -> String;
    fn phone(&self) -> String;
    fn location(&self) -> String;
    fn seller_name(&self) -> String;
    fn equipment(&self) -> String;
    fn seller_url(&self) -> String;
    fn consumption_fuel(&self) -> f32;
    fn consumption_kw(&self) -> f32;
    fn co2(&self) -> u32;
    fn range(&self) -> u32;
    fn days_in_sale(&self) -> Option<u32>;
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
    fn search_id(&self) -> String;
    fn url(&self) -> String;
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
            id: item.id(),
            source: item.source(),
            make,
            model: item.model(),
            title: item.title(),
            currency: item.currency(),
            price: item.price(),
            mileage: item.millage(),
            month: item.month(),
            year: item.year(),
            engine: item.engine(),
            gearbox: item.gearbox(),
            cc: item.cc(),
            power_ps: item.power_ps(),
            power_kw: item.power_kw(),
            search_id: item.search_id(),
            url: item.url(),
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

impl<T> From<T> for DetailedVehicleInfo
where
    T: DetailsT,
{
    fn from(record: T) -> Self {
        Self {
            id: record.get_id(),
            source: record.source(),
            location: record.location(),
            seller_name: record.seller_name(),
            equipment: record.equipment(),
            seller_url: record.seller_url(),
            consumption_fuel: record.consumption_fuel(),
            consumption_kw: record.consumption_kw(),
            co2: record.co2(),
            range: record.range(),
            days_in_sale: record.days_in_sale(),
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
