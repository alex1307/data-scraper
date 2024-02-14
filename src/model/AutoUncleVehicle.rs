use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::config::Equipment::get_equipment_as_u64;

use super::{
    enums::{Currency, Engine, Gearbox},
    traits::{Identity, URLResource},
    VehicleDataModel::{BaseVehicleInfo, DetailedVehicleInfo, Price, VehicleChangeLogInfo},
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
    pub km: Option<u32>,

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
    pub price: Option<u32>,

    //#[serde(rename = "regMonth")]
    pub reg_month: Option<String>,

    #[serde(rename = "sourceName")]
    pub source_name: String,

    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,

    #[serde(rename = "vdpPath")]
    pub vdp_path: String,

    #[serde(rename = "year")]
    pub year: Option<u16>,

    #[serde(rename = "youSaveDifference")]
    pub you_save_difference: Option<u32>,

    #[serde(rename = "laytime")]
    pub laytime: Option<u32>,

    #[serde(rename = "sellerKind")]
    pub seller_kind: String,

    #[serde(rename = "isElectric")]
    pub is_electric: bool,

    #[serde(rename = "priceChange")]
    pub price_change: Option<i32>,

    #[serde(skip)]
    pub source: String,
}

impl From<AutoUncleVehicle> for BaseVehicleInfo {
    fn from(source: AutoUncleVehicle) -> Self {
        let mut base = BaseVehicleInfo::new(source.id.clone());
        if let Some(is_automatic) = source.has_auto_gear {
            base.gearbox = if is_automatic {
                Gearbox::Automatic
            } else {
                Gearbox::Manual
            };
        } else {
            base.gearbox = Gearbox::Manual;
        }

        if let Some(engine_fuel) = source.fuel.clone() {
            base.engine = Engine::from_str(&engine_fuel).unwrap();
        } else {
            base.engine = Engine::NotAvailable;
        }

        if let Some(brand) = source.brand.clone() {
            base.make = brand;
        }

        if let Some(model) = source.car_model.clone() {
            base.model = model;
        }

        if let Some(year) = source.year {
            base.year = year;
        }

        if let Some(hp) = source.hp {
            base.power_ps = hp as u32;
        } else {
            base.power_ps = 0;
        }

        base.currency = source.currency.unwrap_or(Currency::EUR);
        base.price = source.price;
        base.millage = source.km;
        base.source = "autouncle".to_string();
        base
    }
}

impl From<AutoUncleVehicle> for DetailedVehicleInfo {
    fn from(source: AutoUncleVehicle) -> Self {
        let mut details = DetailedVehicleInfo::new(source.id.clone(), 0);
        details.location = source.location.clone();
        if let Some(cc) = source.engine_size {
            details.cc = (cc * 1000.0) as u32;
        }
        if let Some(fuel_consumption) = source.fuel_economy {
            details.fuel_consumption = fuel_consumption;
        }
        if let Some(electric_drive_range) = source.electric_drive_range {
            details.electric_drive_range = electric_drive_range;
        }
        details.is_dealer = source.seller_kind.to_lowercase() == "dealer";
        details.seller_name = source.source_name.clone();
        details.source = "autouncle".to_string();

        let equipment_list = source.featured_attributes_equipment;
        let equipment = get_equipment_as_u64(equipment_list);
        details.equipment = equipment;
        details
    }
}

impl From<AutoUncleVehicle> for VehicleChangeLogInfo {
    fn from(source: AutoUncleVehicle) -> Self {
        let mut log = VehicleChangeLogInfo::new(source.id.clone(), source.source);
        log.days_in_sale = source.laytime;
        if let Some(last_modified) = source.updated_at {
            log.last_modified_on = last_modified;
        }
        log.source = "autouncle".to_string();
        log
    }
}

impl From<AutoUncleVehicle> for Price {
    fn from(source: AutoUncleVehicle) -> Self {
        let mut price = Price::new(source.id.clone(), source.source);
        price.currency = source.currency.unwrap_or(Currency::EUR);
        price.estimated_price = source.estimated_price;
        price.price = source.price.unwrap_or(0);
        price.save_difference = source.you_save_difference.unwrap_or(0);
        price.source = "autouncle".to_string();
        price
    }
}

impl Identity for AutoUncleVehicle {
    fn get_id(&self) -> String {
        self.id.clone()
    }
}

impl URLResource for AutoUncleVehicle {
    fn get_url(&self) -> String {
        format!("https://www.autouncle.dk/en/cars/{}", self.id)
    }
}
