use std::collections::HashMap;
use std::str::FromStr;

use serde::Deserialize;
use serde::Serialize;

use super::enums::Currency;
use crate::model::traits::Header;
use crate::model::traits::Identity;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MobileList {
    pub id: String,
    pub make: String,
    pub model: String,
    pub currency: Currency,
    pub price: u32,
    pub millage: u32,
    pub year: u16,
    pub promoted: bool,
    pub sold: bool,
    pub url: String,
    pub created_on: String,
    pub dealer: String,
}

impl PartialEq for MobileList {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.price == other.price
            && self.promoted == other.promoted
            && self.sold == other.sold
    }
}

impl MobileList {
    pub fn new(
        id: String,
        make: String,
        model: String,
        price: u32,
        currency: Currency,
        created_on: String,
    ) -> Self {
        MobileList {
            id,
            make,
            model,
            price,
            currency,
            sold: false,
            created_on,
            promoted: false,
            millage: 0,
            url: "".to_string(),
            year: 0,
            dealer: "ALL".to_string(),
        }
    }
}

impl Identity for MobileList {
    fn get_id(&self) -> String {
        self.id.clone()
    }
}

impl Header for MobileList {
    fn header() -> Vec<&'static str> {
        vec![
            "id",
            "make",
            "model",
            "currency",
            "price",
            "millage",
            "year",
            "promoted",
            "sold",
            "created_on",
        ]
    }
}

impl FromStr for MobileList {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut values = s.split(',');
        let id = values.next().unwrap().to_string();
        let make = values.next().unwrap().to_string();
        let model = values.next().unwrap().to_string();
        let currency = Currency::from_str(values.next().unwrap()).unwrap();
        let price = values.next().unwrap().parse::<u32>().unwrap();
        let millage = values.next().unwrap().parse::<u32>().unwrap();
        let year = values.next().unwrap().parse::<u16>().unwrap();
        let promoted = values.next().unwrap().parse::<bool>().unwrap();
        let sold = values.next().unwrap().parse::<bool>().unwrap();
        let created_on = values.next().unwrap().to_string();
        let url = values.next().unwrap().to_string();
        let dealer = values.next().unwrap().to_string();
        Ok(MobileList {
            id,
            make,
            model,
            currency,
            price,
            millage,
            year,
            promoted,
            sold,
            created_on,
            url,
            dealer,
        })
    }
}

impl From<HashMap<String, String>> for MobileList {
    fn from(map: HashMap<String, String>) -> Self {
        let default_0 = &"0".to_string();
        let default_str = &"none".to_string();
        let id = map.get("id").unwrap_or(default_str).to_string();
        let make = map.get("make").unwrap_or(default_str).to_string();
        let model = map.get("model").unwrap_or(default_str).to_string();
        let currency = Currency::from_str(map.get("currency").unwrap_or(default_str)).unwrap();
        let price = map
            .get("price")
            .unwrap_or(default_0)
            .parse::<u32>()
            .unwrap();
        let millage = map
            .get("millage")
            .unwrap_or(default_0)
            .parse::<u32>()
            .unwrap();
        let year = map.get("year").unwrap_or(default_0).parse::<u16>().unwrap();
        let promoted = map
            .get("promoted")
            .unwrap_or(&"".to_string())
            .parse::<bool>()
            .unwrap();
        let sold = map
            .get("sold")
            .unwrap_or(&"".to_string())
            .parse::<bool>()
            .unwrap();
        let created_on = map.get("created_on").unwrap_or(default_str).to_string();
        let url = map.get("url").unwrap_or(default_str).to_string();
        let dealer = map.get("dealer").unwrap_or(default_str).to_string();
        MobileList {
            id,
            make,
            model,
            currency,
            price,
            millage,
            year,
            promoted,
            sold,
            created_on,
            url,
            dealer,
        }
    }
}
