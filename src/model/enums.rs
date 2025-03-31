use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

use super::VehicleDataModel::{BaseVehicleInfo, DetailedVehicleInfo, Price};

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize, Default)]
pub enum Currency {
    #[serde(rename = "BGN")]
    BGN,

    #[default]
    #[serde(rename = "EUR")]
    EUR,

    #[serde(rename = "USD")]
    USD,
    #[serde(rename = "CHF")]
    CHF,
    #[serde(rename = "PLN")]
    PLN,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize, Default)]
pub enum Dealer {
    #[serde(rename = "Private")]
    PRIVATE,
    #[serde(rename = "Dealer")]
    DEALER,

    #[default]
    #[serde(rename = "All")]
    ALL,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize, Default)]
pub enum SaleType {
    SOLD,
    INSALE,
    #[default]
    NONE,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize, Default)]
pub enum Engine {
    #[serde(rename = "Petrol")]
    Petrol,
    #[serde(rename = "Diesel")]
    Diesel,
    #[serde(rename = "Hybrid")]
    Hybrid,
    #[serde(rename = "LPG")]
    LPG,
    #[serde(rename = "CNG")]
    CNG,
    #[serde(rename = "HybridPetrol")]
    HybridPetrol,

    #[serde(rename = "HybridDiesel")]
    HybridDiesel,

    #[serde(rename = "Electric")]
    Electric,

    #[serde(rename = "PlugInHybridPetrol")]
    PlugInHybridPetrol,

    #[serde(rename = "PlugInHybridDiesel")]
    PlugInHybridDiesel,

    #[serde(rename = "PluginHybrid")]
    PluginHybrid,

    #[default]
    #[serde(rename = "N/A")]
    NotAvailable,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize, Default)]
pub enum Gearbox {
    #[serde(rename = "Automatic")]
    Automatic,
    #[serde(rename = "Manual")]
    Manual,
    #[serde(rename = "Semi-automatic")]
    Semiautomatic,
    #[default]
    #[serde(rename = "N/A")]
    NotAvailable,
}

#[derive(Debug, Clone)]
pub enum Payload<T> {
    Empty,
    Data(Vec<T>),
    Value(T),
    Error(T),
    Done,
}

impl Display for Gearbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Gearbox::Automatic => "Automatic",
            Gearbox::Manual => "Manual",
            Gearbox::Semiautomatic => "Semi-automatic",
            Gearbox::NotAvailable => "NotFound",
        };
        write!(f, "{}", s)
    }
}

impl Display for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Engine::Petrol => write!(f, "Petrol"),
            Engine::Diesel => write!(f, "Diesel"),
            Engine::Hybrid => write!(f, "Hybrid"),
            Engine::LPG => write!(f, "LPG"),
            Engine::CNG => write!(f, "CNG"),
            Engine::HybridPetrol => write!(f, "HybridPetrol"),
            Engine::HybridDiesel => write!(f, "HybridDiesel"),
            Engine::Electric => write!(f, "Electric"),
            Engine::PlugInHybridPetrol => write!(f, "PlugInHybridPetrol"),
            Engine::PlugInHybridDiesel => write!(f, "PlugInHybridDiesel"),
            Engine::PluginHybrid => write!(f, "PluginHybrid"),
            Engine::NotAvailable => write!(f, "NotAvailable"),
        }
    }
}

impl FromStr for Gearbox {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "автоматична" => Ok(Gearbox::Automatic),
            "автоматични скорости" => Ok(Gearbox::Automatic),
            "automatic" => Ok(Gearbox::Automatic),
            "automatik" => Ok(Gearbox::Automatic),
            "ръчна" => Ok(Gearbox::Manual),
            "schaltgetriebe" => Ok(Gearbox::Manual),
            "manual gearbox" => Ok(Gearbox::Manual),
            "manual" => Ok(Gearbox::Manual),
            "ръчни скорости" => Ok(Gearbox::Manual),
            "полуавтоматична" => Ok(Gearbox::Semiautomatic),
            "semiauto" => Ok(Gearbox::Semiautomatic),
            _ => Err("not found".to_string()),
        }
    }
}

impl FromStr for Engine {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "бензинов" => Ok(Engine::Petrol),
            "petrol" => Ok(Engine::Petrol),
            "бензин" => Ok(Engine::Petrol),
            "benzin" => Ok(Engine::Petrol),
            "газ/бензин" => Ok(Engine::LPG),
            "gas/lpg" => Ok(Engine::LPG),
            "lpg" => Ok(Engine::LPG),
            "lpg_hybrid" => Ok(Engine::LPG),
            "метан/бензин" => Ok(Engine::CNG),
            "natural gas(cng)" => Ok(Engine::CNG),
            "дизелов" => Ok(Engine::Diesel),
            "дизел" => Ok(Engine::Diesel),
            "diesel" => Ok(Engine::Diesel),
            "plug-in хибрид" => Ok(Engine::PluginHybrid),
            "electric" => Ok(Engine::Electric),
            "електрически" => Ok(Engine::Electric),
            "електричество" => Ok(Engine::Electric),
            "хибриден" => Ok(Engine::Hybrid),
            "hybrid" => Ok(Engine::Hybrid),
            "el_hybrid" => Ok(Engine::Hybrid),
            "el_benzin" => Ok(Engine::HybridPetrol),
            "хибрид" => Ok(Engine::Hybrid),
            "hybrid petrol" => Ok(Engine::HybridPetrol),
            "el_diesel" => Ok(Engine::HybridDiesel),
            "hybrid diesel" => Ok(Engine::HybridDiesel),
            "plug-in hybrid petrol" => Ok(Engine::PlugInHybridPetrol),
            "hybrid (petrol/electric)" => Ok(Engine::PlugInHybridPetrol),
            "hybrid (diesel/electric)" => Ok(Engine::PlugInHybridDiesel),
            "plug-in hybrid diesel" => Ok(Engine::PlugInHybridDiesel),
            "el" => Ok(Engine::Electric),
            "cng_hybrid" => Ok(Engine::Hybrid),
            "ethanol_benzin" => Ok(Engine::Petrol),
            _ => Err("not found".to_string()),
        }
    }
}

impl Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Currency::BGN => "BGN",
            Currency::EUR => "EUR",
            Currency::USD => "USD",
            Currency::CHF => "CHF",
            Currency::PLN => "PLN",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for Currency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BGN" => Ok(Currency::BGN),
            "EUR" => Ok(Currency::EUR),
            "USD" => Ok(Currency::USD),
            "CHF" => Ok(Currency::CHF),
            "PLN" => Ok(Currency::PLN),
            _ => Err(format!("Invalid currency code: {}", s)),
        }
    }
}

impl Display for SaleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SaleType::INSALE => "INSALE",
            SaleType::SOLD => "SOLD",
            SaleType::NONE => "NONE",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for SaleType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "INSALE" => Ok(SaleType::INSALE),
            "SOLD" => Ok(SaleType::SOLD),
            _ => Ok(SaleType::NONE),
        }
    }
}

pub enum Message<S: Clone + Serialize + Send + 'static> {
    Message(S),
    Done,
    Error(String),
}

pub enum MessageType {
    BaseVehicleInfo(BaseVehicleInfo),
    DetailedVehicleInfo(DetailedVehicleInfo),
    PriceCalculator(Price),
}
