use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{
    helpers::{
        CURRENCY_KEY, DEALER_KEY, ENGINE_KEY, EQUIPMENT_KEY, GEARBOX_KEY, LOCATION_KEY, MAKE_KEY,
        MILEAGE_KEY, MODEL_KEY, PHONE_KEY, POWER_KEY, PRICE_KEY, PUBLISHED_ON_KEY, SOLD_KEY,
        TOP_KEY, VIEW_COUNT_KEY, VIP_KEY, YEAR_KEY,
    },
    CREATED_ON,
};

use super::{
    enums::{Currency, Engine, Gearbox},
    traits::{Header, Identity},
};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MobileRecord {
    pub id: String,
    pub title: String,
    pub source: String,
    pub make: String,
    pub model: String,
    pub currency: Currency,
    pub price: u32,
    pub millage: u32,
    pub year: u16,
    pub engine: Engine,
    pub gearbox: Gearbox,
    pub power: u32,
    pub phone: String,
    pub location: String,
    pub view_count: u32,
    pub equipment: u64,
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

impl From<HashMap<String, String>> for MobileRecord {
    fn from(map: HashMap<String, String>) -> Self {
        let default_0 = &"0".to_owned();
        let default_str = &"none".to_owned();
        let id = map.get("id").unwrap_or(default_str).to_owned();
        let phone = map
            .get(&PHONE_KEY.to_owned())
            .unwrap_or(default_str)
            .to_string();
        let engine = Engine::from_str(
            map.get(&ENGINE_KEY.to_owned())
                .unwrap_or(default_str)
                .as_str(),
        )
        .unwrap_or(Engine::NotAvailable);
        let gearbox = Gearbox::from_str(
            map.get(&GEARBOX_KEY.to_owned())
                .unwrap_or(default_str)
                .as_str(),
        )
        .unwrap_or(Gearbox::NotAvailable);
        let power = map
            .get(&POWER_KEY.to_owned())
            .unwrap_or(default_0)
            .parse::<u32>()
            .unwrap_or(0);
        let view_count = map
            .get(&VIEW_COUNT_KEY.to_owned())
            .unwrap_or(default_0)
            .parse::<u32>()
            .unwrap_or(0);
        let equipment = map
            .get(&EQUIPMENT_KEY.to_owned())
            .unwrap_or(default_0)
            .parse::<u64>()
            .unwrap_or(0);
        let price = map
            .get(&PRICE_KEY.to_owned())
            .unwrap_or(default_0)
            .parse::<u32>()
            .unwrap_or(0);
        let currency = Currency::from_str(
            map.get(&CURRENCY_KEY.to_owned())
                .unwrap_or(default_str)
                .as_str(),
        )
        .unwrap_or(Currency::BGN);
        let millage = map
            .get(&MILEAGE_KEY.to_owned())
            .unwrap_or(default_0)
            .parse::<u32>()
            .unwrap();
        let year = map
            .get(&YEAR_KEY.to_owned())
            .unwrap_or(default_0)
            .parse::<u16>()
            .unwrap_or(0);
        let top = map
            .get(&TOP_KEY.to_owned())
            .unwrap_or(&"false".to_string())
            .parse::<bool>()
            .unwrap();
        let vip = map
            .get(&VIP_KEY.to_owned())
            .unwrap_or(&"false".to_string())
            .parse::<bool>()
            .unwrap();
        let dealer = map
            .get(&DEALER_KEY.to_owned())
            .unwrap_or(&"false".to_string())
            .parse::<bool>()
            .unwrap_or(false);
        let sold = map
            .get(&SOLD_KEY.to_owned())
            .unwrap_or(&"false".to_string())
            .parse::<bool>()
            .unwrap();
        let location = map
            .get(&LOCATION_KEY.to_string())
            .unwrap_or(default_str)
            .to_string();
        let make = map
            .get(&MAKE_KEY.to_string())
            .unwrap_or(default_str)
            .to_string();
        let model = map
            .get(&MODEL_KEY.to_string())
            .unwrap_or(default_str)
            .to_string();
        let updated_on = map
            .get(&PUBLISHED_ON_KEY.to_string())
            .unwrap_or(default_str)
            .to_string();
        let source = map.get("source").unwrap_or(default_str).to_string();
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
            top,
            vip,
            sold,
            dealer,
            make,
            model,
            created_on: CREATED_ON.to_string(),
            updated_on,
            source,
            ..Default::default()
        }
    }
}
