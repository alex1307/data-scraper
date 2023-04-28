use std::{collections::HashMap, str::FromStr};

use super::{
    enums::{Currency, Engine, Gearbox},
    traits::{Header, Identity},
};

use chrono::Local;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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

impl Identity for MobileDetails {
    fn get_id(&self) -> String {
        self.id.clone()
    }
}

impl Header for MobileDetails {
    fn header() -> Vec<&'static str> {
        vec![
            "id",
            "engine",
            "gearbox",
            "power",
            "phone",
            "view_count",
            "equipment",
            "created_on",
        ]
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

impl From<HashMap<String, String>> for MobileDetails {
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
        let created_on = map.get("created_on").unwrap_or(default_str).to_string();
        let price = map
            .get("price")
            .unwrap_or(default_0)
            .parse::<u32>()
            .unwrap_or(0);
        let currency = Currency::from_str(map.get("currency").unwrap_or(default_str).as_str())
            .unwrap_or(Currency::BGN);
        MobileDetails {
            id,
            phone,
            engine,
            gearbox,
            power,
            view_count,
            equipment,
            created_on,
            price,
            currency,
            extras: vec![],
        }
    }
}
