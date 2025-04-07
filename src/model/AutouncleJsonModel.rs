use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

use super::{
    VehicleDataModel::{BasicT, DetailsT, PriceT},
    enums::{Currency, Engine, Gearbox},
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
    pub displayable_images: String,
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
        if !self.price.is_empty() {
            //get only numbers from the string
            if let Some(captures) = self
                .price
                .chars()
                .filter(|c| c.is_numeric())
                .collect::<String>()
                .parse::<u32>()
                .ok()
            {
                return Some(captures);
            }
        }
        None
    }

    fn estimated_price(&self) -> Option<u32> {
        if let Some(captures) = &self
            .modal_price_history_values
            .estimated_price
            .chars()
            .filter(|c| c.is_numeric())
            .collect::<String>()
            .parse::<u32>()
            .ok()
        {
            Some(*captures)
        } else {
            None
        }
    }

    fn save_difference(&self) -> u32 {
        if let Some(captures) = &self
            .modal_price_history_values
            .you_save
            .chars()
            .filter(|c| c.is_numeric())
            .collect::<String>()
            .parse::<u32>()
            .ok()
        {
            *captures
        } else {
            0
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

impl BasicT for CarData {
    fn id(&self) -> String {
        self.car_id.clone()
    }
    fn source(&self) -> String {
        self.source.clone()
    }
    fn price(&self) -> Option<u32> {
        self.price()
    }
    fn currency(&self) -> Currency {
        self.currency()
    }
    fn make(&self) -> String {
        self.brand.clone()
    }
    fn model(&self) -> String {
        self.car_model.clone()
    }
    fn year(&self) -> u16 {
        self.year.unwrap_or(0) as u16
    }
    fn power_ps(&self) -> u32 {
        self.hp().unwrap_or(0)
    }
    fn gearbox(&self) -> Gearbox {
        if self.has_auto_gear {
            Gearbox::Automatic
        } else {
            Gearbox::Manual
        }
    }
    fn engine(&self) -> Engine {
        self.engine
    }

    fn millage(&self) -> Option<u32> {
        self.km
    }
    fn cc(&self) -> u32 {
        if let Some(cc) = self.engine_size {
            (cc * 1000.0) as u32
        } else {
            0
        }
    }
    fn power_kw(&self) -> u32 {
        self.kw().unwrap_or(0)
    }
    fn month(&self) -> Option<u16> {
        None
    }
    fn title(&self) -> String {
        self.headline.clone()
    }
    fn search_id(&self) -> String {
        self.searchId.clone()
    }
    fn url(&self) -> String {
        if let Some(url) = &self.outgoing_path {
            match self.source.as_str() {
                "autouncle.ro" => format!("https://www.autouncle.ro{}", url),
                "autouncle.fr" => format!("https://www.autouncle.fr{}", url),
                "autouncle.nl" => format!("https://www.autouncle.nl{}", url),
                "autouncle.ch" => format!("https://www.autouncle.ch{}", url),
                "autouncle.pl" => format!("https://www.autouncle.pl{}", url),
                _ => "".to_string(),
            }
        } else {
            "".to_string()
        }
    }
}

impl DetailsT for CarData {
    fn get_id(&self) -> String {
        self.car_id.clone()
    }

    fn source(&self) -> String {
        self.source.clone()
    }

    fn phone(&self) -> String {
        "".to_string()
    }

    fn location(&self) -> String {
        self.location.clone().unwrap_or_default()
    }

    fn seller_name(&self) -> String {
        self.source_name.clone()
    }

    fn equipment(&self) -> String {
        if let Some(equipment) = &self.modal_price_history_values.car_equipment {
            equipment.clone()
        } else {
            "".to_string()
        }
    }

    fn seller_url(&self) -> String {
        "".to_string()
    }

    fn consumption_fuel(&self) -> f32 {
        self.litter_fuel_consumption().unwrap_or(0.0)
    }

    fn consumption_kw(&self) -> f32 {
        self.kwh_fuel_consumption().unwrap_or(0.0)
    }

    fn co2(&self) -> u32 {
        self.co2_emission().unwrap_or(0) as u32
    }

    fn range(&self) -> u32 {
        self.range().unwrap_or(0) as u32
    }

    fn days_in_sale(&self) -> Option<u32> {
        self.laytime
    }
}

impl PriceT for CarData {
    fn currency(&self) -> Currency {
        self.currency()
    }

    fn price(&self) -> u32 {
        self.price().unwrap_or(0)
    }

    fn id(&self) -> String {
        self.car_id.clone()
    }

    fn source(&self) -> String {
        self.source.clone()
    }

    fn estimated_price(&self) -> Option<u32> {
        self.estimated_price()
    }

    fn save_difference(&self) -> u32 {
        self.save_difference()
    }

    fn overpriced_difference(&self) -> u32 {
        0
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
