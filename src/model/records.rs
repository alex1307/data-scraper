use serde::{Deserialize, Serialize};

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
    pub promoted: bool,
    pub sold: bool,
    pub engine: Engine,
    pub gearbox: Gearbox,
    pub power: u16,
    pub phone: String,
    pub view_count: u32,
    #[serde(skip)]
    pub extras: Vec<String>,
    pub equipment: u64,
    pub created_on: String,
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
            "promoted",
            "sold",
            "dealer",
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

impl Identity for MobileRecord {
    fn get_id(&self) -> String {
        self.id.clone()
    }
}
