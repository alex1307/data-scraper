use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::helpers::{
    CURRENCY_KEY, DEALER_KEY, ENGINE_KEY, GEARBOX_KEY, MAKE_KEY, MILEAGE_KEY, MODEL_KEY, POWER_KEY,
    PRICE_KEY,
};

use super::{
    enums::{Currency, Engine, Gearbox},
    traits::Identity,
    VehicleDataModel::{BaseVehicleInfo, DetailedVehicleInfo, VehicleChangeLogInfo},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AutoUncleVehicle {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "announcedAsNew")]
    pub announced_as_new: bool,

    #[serde(rename = "auRating")]
    pub au_rating: Option<u8>,

    #[serde(rename = "brand")]
    pub brand: Option<String>,

    #[serde(rename = "body")]
    pub body: Option<String>,

    #[serde(rename = "displayableFuelConsumption")]
    pub displayable_fuel_consumption: Option<String>,

    #[serde(rename = "carModel")]
    pub car_model: Option<String>,

    #[serde(rename = "co2Emission")]
    pub co2_emission: Option<f64>,

    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,

    #[serde(rename = "currency")]
    pub currency: Option<Currency>,

    #[serde(rename = "doors")]
    pub doors: Option<u8>,

    #[serde(rename = "electricDriveRange")]
    pub electric_drive_range: Option<f64>,

    #[serde(rename = "engineSize")]
    pub engine_size: Option<f64>,

    #[serde(rename = "equipmentVariant")]
    pub equipment_variant: Option<String>,

    #[serde(rename = "estimatedPrice")]
    pub estimated_price: Option<u32>,

    #[serde(rename = "featuredAttributesEquipment")]
    pub featured_attributes_equipment: Vec<String>,

    #[serde(rename = "featuredAttributesNonEquipment")]
    pub featured_attributes_non_equipment: Vec<String>,

    #[serde(rename = "fuel")]
    pub fuel: Option<String>,

    #[serde(rename = "fuelEconomy")]
    pub fuel_economy: Option<f64>,

    #[serde(rename = "hasAutoGear")]
    pub has_auto_gear: Option<bool>,

    #[serde(rename = "headline")]
    pub headline: Option<String>,

    #[serde(rename = "hp")]
    pub hp: Option<u16>,

    #[serde(rename = "isFeatured")]
    pub is_featured: Option<bool>,

    #[serde(rename = "km")]
    pub km: u32,

    #[serde(rename = "kw")]
    pub kw: Option<u16>,

    #[serde(rename = "localizedFuelEconomy")]
    pub localized_fuel_economy: Option<f64>,

    #[serde(rename = "localizedFuelEconomyLabel")]
    pub localized_fuel_economy_label: String,

    #[serde(rename = "location")]
    pub location: String,

    #[serde(rename = "modelGeneration")]
    pub model_generation: String,

    #[serde(rename = "noRatingReasons")]
    pub no_rating_reasons: Vec<String>,

    #[serde(rename = "outgoingPath")]
    pub outgoing_path: String,

    #[serde(rename = "price")]
    pub price: u32,

    #[serde(rename = "regMonth")]
    pub reg_month: Option<u8>,

    #[serde(rename = "sourceName")]
    pub source_name: String,

    #[serde(rename = "updatedAt")]
    pub updated_at: String,

    #[serde(rename = "vdpPath")]
    pub vdp_path: String,

    #[serde(rename = "year")]
    pub year: Option<u16>,

    #[serde(rename = "youSaveDifference")]
    pub you_save_difference: u32,

    #[serde(rename = "laytime")]
    pub laytime: u32,

    #[serde(rename = "sellerKind")]
    pub seller_kind: String,

    #[serde(rename = "isElectric")]
    pub is_electric: bool,

    #[serde(rename = "priceChange")]
    pub price_change: Option<i32>,
}

impl AutoUncleVehicle {
    pub fn to_vehicle_details(&self) -> DetailedVehicleInfo {
        let mut details = DetailedVehicleInfo::new(self.id.clone(), 0);
        details.location = self.location.clone();
        if let Some(cc) = self.engine_size {
            details.cc = cc as u32;
        }
        if let Some(fuel_consumption) = self.fuel_economy {
            details.fuel_consumption = fuel_consumption;
        }
        if let Some(electric_drive_range) = self.electric_drive_range {
            details.electric_drive_range = electric_drive_range;
        }

        details
    }

    pub fn to_vehicle_change_log_info(&self) -> VehicleChangeLogInfo {
        let mut log = VehicleChangeLogInfo::new(self.id.clone(), self.updated_at.clone());
        log.days_in_sale = self.laytime;
        log.last_modified_on = self.updated_at.clone();
        log
    }
    pub fn to_base(&self) -> BaseVehicleInfo {
        let mut base = BaseVehicleInfo::new(self.id.clone());
        if let Some(is_automatic) = self.has_auto_gear {
            base.gearbox = if is_automatic {
                Gearbox::Automatic
            } else {
                Gearbox::Manual
            };
        } else {
            base.gearbox = Gearbox::Manual;
        }

        if let Some(engine_fuel) = self.fuel.clone() {
            base.engine = Engine::from_str(&engine_fuel).unwrap();
        } else {
            base.engine = Engine::NotAvailable;
        }

        if let Some(brand) = self.brand.clone() {
            base.make = brand;
        } else {
            base.make = "N/A".to_string();
        }

        if let Some(model) = self.car_model.clone() {
            base.model = model;
        } else {
            base.model = "N/A".to_string();
        }

        if let Some(year) = self.year {
            base.year = year;
        } else {
            base.year = 0;
        }

        if let Some(hp) = self.hp {
            base.power = hp;
        } else {
            base.power = 0;
        }

        base.currency = self.currency.unwrap_or(Currency::EUR);
        base.price = self.price;
        base.millage = self.km;
        base
    }

    pub fn to_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("id".to_string(), self.id.clone());
        let is_automatic = self.has_auto_gear.unwrap_or(false);

        if self.fuel.is_none() {
            if !self.is_electric {
                return map;
            }
            map.insert(ENGINE_KEY.to_string(), Engine::Electric.to_string());
        } else {
            let engine = self.fuel.clone().unwrap();
            map.insert(ENGINE_KEY.to_string(), engine);
        }

        if self.brand.is_none() {
            return map;
        } else {
            let brand = self.brand.clone().unwrap();
            map.insert(MAKE_KEY.to_string(), brand);
        }

        if self.car_model.is_none() {
            return map;
        } else {
            let model = self.car_model.clone().unwrap();
            map.insert(MODEL_KEY.to_string(), model);
        }

        if self.year.is_none() {
            return map;
        } else {
            let year = self.year.clone().unwrap();
            map.insert("year".to_string(), year.to_string());
        }

        if is_automatic {
            map.insert(GEARBOX_KEY.to_owned(), Gearbox::Automatic.to_string());
        } else {
            map.insert(GEARBOX_KEY.to_owned(), Gearbox::Manual.to_string());
        }

        let is_dealer = self.seller_kind.trim().to_lowercase() == "dealer";
        map.insert(DEALER_KEY.to_string(), is_dealer.to_string());

        if self.currency.is_none() {
            return map;
        } else {
            let currency = self.currency.clone().unwrap();
            map.insert(CURRENCY_KEY.to_string(), currency.to_string());
        }

        map.insert(PRICE_KEY.to_string(), self.price.to_string());
        if self.km > 0 {
            map.insert(MILEAGE_KEY.to_string(), self.km.to_string());
        }
        if let Some(hp) = self.hp {
            map.insert(POWER_KEY.to_string(), hp.to_string());
        }

        if self.laytime > 0 {
            map.insert("laytime".to_string(), self.laytime.to_string());
        }

        map
    }
}

impl Identity for AutoUncleVehicle {
    fn get_id(&self) -> String {
        self.id.clone()
    }
}
