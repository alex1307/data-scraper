use lazy_static::lazy_static;
use log::error;
use regex::Regex;
use serde::{Deserialize, Serialize};

use super::{
    enums::{Currency, Engine, Gearbox},
    traits::VehicleT,
};
lazy_static! {
    static ref STRING_TO_F32: Regex = Regex::new(r"\d+(\.\d+)").unwrap();
    static ref STRING_TO_I32: Regex = Regex::new(r"\d+").unwrap();
    static ref RANGE_TO_I32: Regex = Regex::new(r"≈\s*(\d+)\s*km").unwrap();
    static ref HP_REGEX: Regex = Regex::new(r"(\d+)\s*HP").unwrap();
    static ref KW_REGEX: Regex = Regex::new(r"(\d+)\s*kW").unwrap();
    pub static ref DIESEL_ENGINE_REGEX: Regex = Regex::new(r"\d+(\.\d+)L\s*Diesel").unwrap();
    pub static ref PETROL_ENGINE_REGEX: Regex = Regex::new(r"\d+(\.\d+)L\s*Petrol").unwrap();
    pub static ref HYBRID_ENGINE_REGEX: Regex = Regex::new(r"\d+(\.\d+)L\s*Hybrid").unwrap();
    pub static ref ELECTRIC_ENGINE_REGEX: Regex = Regex::new(r"\s*Electric").unwrap();
    pub static ref LPG_ENGINE_REGEX: Regex = Regex::new(r"\d+(\.\d+)L\s*LPG").unwrap();
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CarData {
    pub car_id: String,
    pub vdp_path: String,
    pub source_name: String,
    pub headline: String,
    pub fuel_consumption: Option<String>,
    pub co2_emission: Option<String>,
    pub range: Option<String>,
    pub index: Option<u32>,
    pub lang: String,
    pub login_url: String,
    pub km: Option<u32>,
    pub brand: String,
    pub car_model: String,
    pub model_generation: Option<String>,
    pub equipment_variant: Option<String>,
    pub laytime: Option<u32>,
    pub is_featured: Option<bool>,
    pub year: Option<u32>,
    pub doors: Option<u32>,
    pub body: Option<String>,
    pub has_auto_gear: bool,
    pub engine_size: Option<f32>,
    pub country_default_locale: String,
    pub country_currency_code: Option<String>,
    pub available_for_online_sales: Option<bool>,
    pub online_sales_tooltip: Option<String>,
    pub online_sales_highlight: Option<String>,
    pub outgoing_path: Option<String>,
    pub is_paid_click: bool,
    pub is_private_car: bool,
    pub price: String,
    pub price_change: Option<i32>,
    pub is_verified_dealer: bool,
    pub location: Option<String>,
    pub modal_price_history_values: ModalPriceHistoryValues,

    #[serde(skip)]
    pub source: String,

    #[serde(skip)]
    pub equipment: Vec<String>,

    #[serde(skip)]
    pub searchId: String,

    #[serde(skip)]
    pub engine: Engine,
}

impl CarData {
    pub fn litter_fuel_consumption(&self) -> Option<f32> {
        if let Some(fuel_consumption) = &self.fuel_consumption {
            if fuel_consumption.to_lowercase().contains("l/100") {
                if let Some(captures) = STRING_TO_F32.captures(fuel_consumption) {
                    return Some(captures.get(0).unwrap().as_str().parse::<f32>().unwrap());
                }
            }
        }
        None
    }

    pub fn kwh_fuel_consumption(&self) -> Option<f32> {
        if let Some(fuel_consumption) = &self.fuel_consumption {
            if fuel_consumption.to_lowercase().contains("kwh/100") {
                if let Some(captures) = STRING_TO_F32.captures(fuel_consumption) {
                    return Some(captures.get(0).unwrap().as_str().parse::<f32>().unwrap());
                }
            }
        }
        None
    }

    pub fn co2_emission(&self) -> Option<i32> {
        if let Some(co2_emission) = &self.co2_emission {
            if let Some(captures) = STRING_TO_I32.captures(co2_emission) {
                return Some(captures.get(0).unwrap().as_str().parse::<i32>().unwrap());
            }
        }
        None
    }

    pub fn range(&self) -> Option<u32> {
        if let Some(range) = &self.range {
            if let Some(captures) = RANGE_TO_I32.captures(range) {
                return Some(captures.get(1).unwrap().as_str().parse::<u32>().unwrap());
            }
        }
        None
    }

    pub fn price(&self) -> Option<u32> {
        if let Ok(captures) = &self
            .price
            .chars()
            .filter(|c| c.is_numeric())
            .collect::<String>()
            .parse::<u32>()
        {
            Some(*captures)
        } else {
            None
        }
    }

    fn estimated_price(&self) -> Option<u32> {
        if let Ok(captures) = &self
            .modal_price_history_values
            .estimated_price
            .chars()
            .filter(|c| c.is_numeric())
            .collect::<String>()
            .parse::<u32>()
        {
            Some(*captures)
        } else {
            None
        }
    }

    fn currency(&self) -> Currency {
        match self
            .country_currency_code
            .clone()
            .unwrap_or_default()
            .to_uppercase()
            .as_str()
        {
            "USD" => Currency::USD,
            "CHF" => Currency::CHF,
            "PLN" => Currency::PLN,
            _ => Currency::EUR,
        }
    }

    fn hp(&self) -> Option<u32> {
        if !&self
            .modal_price_history_values
            .displayable_engine_power
            .is_empty()
        {
            if let Some(captures) =
                HP_REGEX.captures(&self.modal_price_history_values.displayable_engine_power)
            {
                return Some(captures.get(1).unwrap().as_str().parse::<u32>().unwrap());
            }
        }
        None
    }

    fn kw(&self) -> Option<u32> {
        if !&self
            .modal_price_history_values
            .displayable_engine_power
            .is_empty()
        {
            if let Some(captures) =
                KW_REGEX.captures(&self.modal_price_history_values.displayable_engine_power)
            {
                return Some(captures.get(1).unwrap().as_str().parse::<u32>().unwrap());
            }
        }
        None
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModalPriceHistoryValues {
    pub car_index: u32,
    pub image_url: String,
    pub car_equipment: Option<String>,
    pub displayable_year: String,
    pub displayable_engine_power: String,
    pub headline: String,
    pub estimated_price: String,
    pub you_save: String,
}

impl VehicleT for CarData {
    fn id(&self) -> String {
        self.car_id.clone()
    }

    fn source(&self) -> String {
        self.source.clone()
    }

    fn make(&self) -> String {
        self.brand.clone()
    }

    fn model(&self) -> String {
        self.car_model.clone()
    }

    fn title(&self) -> String {
        self.headline.clone()
    }

    fn currency(&self) -> Currency {
        self.currency()
    }

    fn price(&self) -> u32 {
        self.price().unwrap_or(0)
    }

    fn mileage(&self) -> u32 {
        self.km.unwrap_or(0)
    }

    fn year(&self) -> u16 {
        self.year.unwrap_or(0) as u16
    }

    fn engine(&self) -> Engine {
        self.engine
    }

    fn gearbox(&self) -> Gearbox {
        if self.has_auto_gear {
            Gearbox::Automatic
        } else {
            Gearbox::Manual
        }
    }

    fn power_ps(&self) -> u32 {
        self.hp().unwrap_or(0)
    }

    fn power_kw(&self) -> u32 {
        self.kw().unwrap_or(0)
    }

    fn cc(&self) -> Option<u32> {
        self.engine_size.map(|cc| (cc * 1000.0) as u32)
    }

    fn url(&self) -> String {
        let url = match self.source.as_str() {
            "autouncle.ro" => format!("https://www.autouncle.ro{}", self.vdp_path),
            "autouncle.fr" => format!("https://www.autouncle.fr{}", self.vdp_path),
            "autouncle.it" => format!("https://www.autouncle.it{}", self.vdp_path),
            "autouncle.nl" => format!("https://www.autouncle.nl{}", self.vdp_path),
            "autouncle.ch" => format!("https://www.autouncle.ch{}", self.vdp_path),
            "autouncle.pl" => format!("https://www.autouncle.pl{}", self.vdp_path),
            "autouncle.de" => format!("https://www.autouncle.de{}", self.vdp_path),
            _ => {
                error!("Unknown source: {}", self.source);
                "".to_string()
            }
        };
        url
    }

    fn location(&self) -> Option<String> {
        self.location.clone()
    }

    fn seller_name(&self) -> Option<String> {
        Some(self.source_name.clone())
    }

    fn equipment(&self) -> Option<String> {
        self.modal_price_history_values.car_equipment.clone()
    }

    fn seller_url(&self) -> Option<String> {
        None
    }

    fn consumption_fuel(&self) -> Option<f32> {
        self.litter_fuel_consumption()
    }

    fn consumption_kw(&self) -> Option<f32> {
        self.kwh_fuel_consumption()
    }

    fn co2(&self) -> Option<u32> {
        self.co2_emission().map(|value| value as u32)
    }

    fn range(&self) -> Option<u32> {
        self.range()
    }

    fn days_in_sale(&self) -> Option<u32> {
        self.laytime
    }

    fn estimated_price(&self) -> Option<u32> {
        self.estimated_price()
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
