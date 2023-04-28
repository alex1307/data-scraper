use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize};

use super::enums::{Currency, Engine, Gearbox};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MobileData {
    pub id: String,
    pub make: String,
    pub model: String,
    pub currency: Currency,
    pub price: u32,
    pub millage: u32,
    pub year: u16,
    pub promoted: bool,
    pub sold: bool,
    pub dealer: String,
    pub engine: Engine,
    pub gearbox: Gearbox,
    pub power: u16,
    pub phone: String,
    pub view_count: u32,
    pub equipment: u64,
    pub created_on: String,
}

impl From<HashMap<String, String>> for MobileData {
    fn from(value: HashMap<String, String>) -> Self {
        let default_0 = &"0".to_string();
        let default_str = &"none".to_string();
        let id = value.get("id").unwrap_or(default_str).to_string();
        let make = value.get("make").unwrap_or(default_str).to_string();
        let model = value.get("model").unwrap_or(default_str).to_string();
        let currency = Currency::from_str(value.get("currency").unwrap_or(default_str))
            .unwrap_or(Currency::BGN);
        let price = value
            .get("price")
            .unwrap_or(default_0)
            .parse::<u32>()
            .unwrap_or(0);
        let millage = value
            .get("millage")
            .unwrap_or(default_0)
            .parse::<u32>()
            .unwrap_or(0);
        let year = value
            .get("year")
            .unwrap_or(default_0)
            .parse::<u16>()
            .unwrap_or(0);
        let promoted = value
            .get("promoted")
            .unwrap_or(default_str)
            .parse::<bool>()
            .unwrap_or(false);
        let sold = value
            .get("sold")
            .unwrap_or(&"false".to_string())
            .parse::<bool>()
            .unwrap_or(false);
        let dealer = value.get("dealer").unwrap_or(default_str).to_string();
        let engine = Engine::from_str(value.get("engine").unwrap_or(default_str))
            .unwrap_or(Engine::NotAvailable);
        let gearbox = Gearbox::from_str(value.get("gearbox").unwrap_or(default_str))
            .unwrap_or(Gearbox::NotAvailable);
        let power = value
            .get("power")
            .unwrap_or(default_0)
            .parse::<u16>()
            .unwrap_or(0);
        let view_count = value
            .get("view_count")
            .unwrap_or(default_0)
            .parse::<u32>()
            .unwrap_or(0);
        let equipment = value
            .get("equipment")
            .unwrap_or(default_0)
            .parse::<u64>()
            .unwrap_or(0);
        let phone = value.get("phone").unwrap_or(default_str).to_string();
        let created_on = value.get("created_on").unwrap_or(default_str).to_string();

        MobileData {
            id,
            make,
            model,
            currency,
            price,
            millage,
            year,
            promoted,
            sold,
            dealer,
            engine,
            gearbox,
            power,
            view_count,
            equipment,
            phone,
            created_on,
        }
    }
}
