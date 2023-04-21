use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Currency {
    BGN,
    EUR,
    USD,
}

impl Default for Currency {
    fn default() -> Self {
        Currency::BGN
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Engine {
    Petrol,
    Diesel,
    Hybrid,
    Electric,
    PluginHybrid,
    NotAvailable,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Gearbox {
    Automatic,
    Manual,
    Semiautomatic,
    NotAvailable,
}

impl Default for Engine {
    fn default() -> Self {
        Engine::NotAvailable
    }
}

impl Default for Gearbox {
    fn default() -> Self {
        Gearbox::NotAvailable
    }
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
            Engine::Petrol => "Бензинов".to_string(),
            Engine::Diesel => "Дизелов".to_string(),
            Engine::PluginHybrid => "Plug-in хибрид".to_string(),
            Engine::Electric => "Електрически".to_string(),
            Engine::Hybrid => "Хибриден".to_string(),
            Engine::NotAvailable => "NotFound".to_string(),
        }
    }
}

impl FromStr for Gearbox {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Автоматична" => Ok(Gearbox::Automatic),
            "Ръчна" => Ok(Gearbox::Manual),
            "Полуавтоматична" => Ok(Gearbox::Semiautomatic),
            _ => Ok(Gearbox::NotAvailable),
        }
    }
}

impl FromStr for Engine {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Бензинов" => Ok(Engine::Petrol),
            "Дизелов" => Ok(Engine::Diesel),
            "Plug-in хибрид" => Ok(Engine::PluginHybrid),
            "Електрически" => Ok(Engine::Electric),
            "Хибриден" => Ok(Engine::Hybrid),
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
