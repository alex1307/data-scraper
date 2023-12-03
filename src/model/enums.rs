use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize, Default)]
pub enum Currency {
    #[default]
    #[serde(rename = "BGN")]
    BGN,

    #[serde(rename = "EUR")]
    EUR,

    #[serde(rename = "USD")]
    USD,
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

impl ToString for Gearbox {
    fn to_string(&self) -> String {
        match self {
            Gearbox::Automatic => "Автоматична".to_string(),
            Gearbox::Manual => "Ръчна".to_string(),
            Gearbox::Semiautomatic => "Полуавтоматична".to_string(),
            Gearbox::NotAvailable => "NotFound".to_string(),
        }
    }
}

impl ToString for Engine {
    fn to_string(&self) -> String {
        match self {
            Engine::Petrol => "Petrol".to_string(),
            Engine::Diesel => "Diesel".to_string(),
            Engine::PluginHybrid => "Plug-in-hybrid".to_string(),
            Engine::Electric => "Electric".to_string(),
            Engine::Hybrid => "Hybrid".to_string(),
            Engine::LPG => "LPG".to_string(),
            Engine::CNG => "CNG".to_string(),
            Engine::HybridPetrol => "Hybrid-petrol".to_string(),
            Engine::HybridDiesel => "Hybrid-diesel".to_string(),
            Engine::PlugInHybridPetrol => "Plug-in-hybrid-petrol".to_string(),
            Engine::PlugInHybridDiesel => "Plug-in-hybrid-diesel".to_string(),
            _ => "NotFound".to_string(),
        }
    }
}

impl FromStr for Gearbox {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Автоматична" => Ok(Gearbox::Automatic),
            "Автоматични скорости" => Ok(Gearbox::Automatic),
            "Automatic" => Ok(Gearbox::Automatic),
            "Ръчна" => Ok(Gearbox::Manual),
            "Manual" => Ok(Gearbox::Manual),
            "Ръчни скорости" => Ok(Gearbox::Manual),
            "Полуавтоматична" => Ok(Gearbox::Semiautomatic),
            "semiauto" => Ok(Gearbox::Semiautomatic),
            _ => Ok(Gearbox::NotAvailable),
        }
    }
}

impl FromStr for Engine {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Бензинов" => Ok(Engine::Petrol),
            "Petrol" => Ok(Engine::Petrol),
            "Бензин" => Ok(Engine::Petrol),
            "Газ/Бензин" => Ok(Engine::LPG),
            "Gas/lpg" => Ok(Engine::LPG),
            "Метан/Бензин" => Ok(Engine::CNG),
            "Natural gas(cng)" => Ok(Engine::CNG),
            "Дизелов" => Ok(Engine::Diesel),
            "Дизел" => Ok(Engine::Diesel),
            "Diesel" => Ok(Engine::Diesel),
            "Plug-in хибрид" => Ok(Engine::PluginHybrid),
            "Electric" => Ok(Engine::Electric),
            "Електрически" => Ok(Engine::Electric),
            "Електричество" => Ok(Engine::Electric),
            "Хибриден" => Ok(Engine::Hybrid),
            "Hybrid" => Ok(Engine::Hybrid),
            "Хибрид" => Ok(Engine::Hybrid),
            "Hybrid petrol" => Ok(Engine::HybridPetrol),
            "Hybrid diesel" => Ok(Engine::HybridDiesel),
            "Plug-in hybrid petrol" => Ok(Engine::PlugInHybridPetrol),
            "Plug-in hybrid diesel" => Ok(Engine::PlugInHybridDiesel),
            _ => Ok(Engine::NotAvailable),
        }
    }
}

impl ToString for Currency {
    fn to_string(&self) -> String {
        match self {
            Currency::BGN => "BGN".to_string(),
            Currency::EUR => "EUR".to_string(),
            Currency::USD => "USD".to_string(),
        }
    }
}

impl FromStr for Currency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BGN" => Ok(Currency::BGN),
            "EUR" => Ok(Currency::EUR),
            "USD" => Ok(Currency::USD),
            _ => Err(format!("Invalid currency code: {}", s)),
        }
    }
}

impl ToString for SaleType {
    fn to_string(&self) -> String {
        match self {
            SaleType::INSALE => "INSALE".to_string(),
            SaleType::SOLD => "SOLD".to_string(),
            SaleType::NONE => "NONE".to_string(),
        }
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
