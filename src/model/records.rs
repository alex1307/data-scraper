use std::{collections::HashMap, str::FromStr};


use serde::{Deserialize, Serialize};


use crate::CREATED_ON;

use super::{
    enums::{Currency, Engine, Gearbox},
    traits::{Header, Identity},
};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MobileRecord {
    pub id: String,
    pub make: String,
    pub model: String,
    pub currency: Currency,
    pub price: u32,
    pub millage: u32,
    pub year: u16,
    pub engine: Engine,
    pub gearbox: Gearbox,
    pub power: u16,
    pub phone: String,
    pub location: String,
    pub view_count: u32,
    pub equipment: u64,
    pub promoted: bool,
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
            "make",
            "model",
            "currency",
            "price",
            "millage",
            "year",
            "engine",
            "gearbox",
            "power",
            "phone",
            "location",
            "view_count",
            "equipment",
            "promoted",
            "sold",
            "dealer",
            "created_on",
            "updated_on",
            "deleted_on",
        ]
    }
}

impl Identity for MobileRecord {
    fn get_id(&self) -> String {
        self.id.clone()
    }
}

impl From<HashMap<String, String>> for MobileRecord {
    fn from(map: HashMap<String, String>) -> Self {
        let default_0 = &"0".to_string();
        let default_str = &"none".to_string();
        let id = map.get("id").unwrap_or(default_str).to_string();
        let phone = map.get("phone").unwrap_or(default_str).to_string();
        let engine = Engine::from_str(map.get("engine").unwrap_or(default_str).as_str())
            .unwrap_or(Engine::NotAvailable);
        let gearbox = Gearbox::from_str(map.get("gearbox").unwrap_or(default_str).as_str())
            .unwrap_or(Gearbox::NotAvailable);
        let power = map
            .get("power")
            .unwrap_or(default_0)
            .parse::<u16>()
            .unwrap_or(0);
        let view_count = map
            .get("view_count")
            .unwrap_or(default_0)
            .parse::<u32>()
            .unwrap_or(0);
        let equipment = map
            .get("equipment")
            .unwrap_or(default_0)
            .parse::<u64>()
            .unwrap_or(0);
        let price = map
            .get("price")
            .unwrap_or(default_0)
            .parse::<u32>()
            .unwrap_or(0);
        let currency = Currency::from_str(map.get("currency").unwrap_or(default_str).as_str())
            .unwrap_or(Currency::BGN);
        let millage = map
            .get("millage")
            .unwrap_or(default_0)
            .parse::<u32>()
            .unwrap();
        let year = map
            .get("year")
            .unwrap_or(default_0)
            .parse::<u16>()
            .unwrap_or(0);
        let promoted = map
            .get("promoted")
            .unwrap_or(&"false".to_string())
            .parse::<bool>()
            .unwrap();
        let dealer = map
            .get("dealer")
            .unwrap_or(&"false".to_string())
            .parse::<bool>()
            .unwrap();
        let sold = map
            .get("sold")
            .unwrap_or(&"false".to_string())
            .parse::<bool>()
            .unwrap();
        let location = map.get("location").unwrap_or(default_str).to_string();
        let make = map.get("make").unwrap_or(default_str).to_string();
        let model = map.get("model").unwrap_or(default_str).to_string();

        MobileRecord {
            id,
            engine,
            gearbox,
            currency,
            price,
            power,
            phone,
            location,
            view_count,
            equipment,
            millage,
            year,
            promoted,
            sold,
            dealer,
            make,
            model,
            created_on: CREATED_ON.to_string(),
            ..Default::default()
        }
    }
}
