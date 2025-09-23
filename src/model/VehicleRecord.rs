use serde::{Deserialize, Serialize};

use crate::helpers::{
    CURRENCY_KEY, DEALER_KEY, ENGINE_KEY, EQUIPMENT_KEY, GEARBOX_KEY, LOCATION_KEY, MAKE_KEY,
    MILEAGE_KEY, MODEL_KEY, PHONE_KEY, POWER_KEY, PRICE_KEY, PUBLISHED_ON_KEY, SOLD_KEY, TOP_KEY,
    VIEW_COUNT_KEY, VIP_KEY, YEAR_KEY,
};

use super::{
    enums::{Currency, Engine, Gearbox},
    traits::VehicleT,
    traits::{Header, Identity},
};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MobileRecord {
    pub id: String,
    pub title: String,
    pub source: String,
    pub searchId: String,
    pub make: String,
    pub model: String,
    pub modification: String,
    pub currency: Currency,
    pub price: u32,
    pub old_price: u32,
    pub mileage: u32,
    pub year: u16,
    pub month: u16,
    pub cc: u32,
    pub engine: Engine,
    pub gearbox: Gearbox,
    pub power: u32,
    pub phone: String,
    pub name: String,
    pub location: String,
    pub dealer_url: String,
    pub view_count: u32,
    pub equipment: String,
    pub top: bool,
    pub vip: bool,
    pub sold: bool,
    pub dealer: bool,
    pub created_on: String,
    pub updated_on: String,
    pub deleted_on: String,
}

impl Header for MobileRecord {
    fn header() -> Vec<&'static str> {
        vec![
            "id",
            &MAKE_KEY,
            &MODEL_KEY,
            &CURRENCY_KEY,
            &PRICE_KEY,
            &MILEAGE_KEY,
            &YEAR_KEY,
            &ENGINE_KEY,
            &GEARBOX_KEY,
            &POWER_KEY,
            &PHONE_KEY,
            &LOCATION_KEY,
            &VIEW_COUNT_KEY,
            &EQUIPMENT_KEY,
            &TOP_KEY,
            &VIP_KEY,
            &SOLD_KEY,
            &DEALER_KEY,
            "created_on",
            &PUBLISHED_ON_KEY,
            "deleted_on",
        ]
    }
}

impl Identity for MobileRecord {
    fn get_id(&self) -> String {
        self.id.clone()
    }
}

impl VehicleT for MobileRecord {
    fn id(&self) -> String {
        self.id.clone()
    }
    fn source(&self) -> String {
        self.source.clone()
    }
    fn make(&self) -> String {
        self.make.clone()
    }
    fn model(&self) -> String {
        self.model.clone()
    }
    fn title(&self) -> String {
        self.title.clone()
    }
    fn currency(&self) -> Currency {
        self.currency
    }
    fn price(&self) -> u32 {
        self.price
    }
    fn mileage(&self) -> u32 {
        self.mileage
    }
    fn year(&self) -> u16 {
        self.year
    }
    fn engine(&self) -> Engine {
        self.engine
    }
    fn gearbox(&self) -> Gearbox {
        self.gearbox
    }
    fn power_ps(&self) -> u32 {
        self.power
    }
    fn power_kw(&self) -> u32 {
        self.power
    }
    fn cc(&self) -> Option<u32> {
        Some(self.cc)
    }
    fn url(&self) -> String {
        match self.source.as_str() {
            "mobile.bg" => format!("https://www.mobile.bg/obiava-{}", self.id),
            "cars.bg" => format!("https://www.cars.bg/offer/{}", self.id),
            _ => "".to_string(),
        }
    }
    fn location(&self) -> Option<String> {
        Some(self.location.clone())
    }
    fn seller_name(&self) -> Option<String> {
        Some(self.name.clone())
    }
    fn equipment(&self) -> Option<String> {
        Some(self.equipment.clone())
    }
    fn seller_url(&self) -> Option<String> {
        Some(self.dealer_url.clone())
    }
    fn consumption_fuel(&self) -> Option<f32> {
        Some(0.0)
    }
    fn consumption_kw(&self) -> Option<f32> {
        Some(0.0)
    }
    fn co2(&self) -> Option<u32> {
        Some(0)
    }
    fn range(&self) -> Option<u32> {
        Some(0)
    }
    fn days_in_sale(&self) -> Option<u32> {
        None
    }
    fn estimated_price(&self) -> Option<u32> {
        None
    }
    fn ranges(&self) -> Option<String> {
        None
    }
    fn rating(&self) -> Option<String> {
        None
    }
    fn thresholds(&self) -> Vec<u32> {
        vec![]
    }
}
