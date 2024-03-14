use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::{
    enums::{Currency, Engine, Gearbox},
    traits::URLResource,
    VehicleDataModel::{BasicT, ChangeLogT, DetailsT, PriceT},
};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AutoUncleVehicleTest {
    #[serde(rename = "id")]
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
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

    #[serde(rename = "featuredAttributesEquipment")]
    pub featured_attributes_equipment: String,

    #[serde(rename = "featuredAttributesNonEquipment")]
    pub featured_attributes_non_equipment: String,

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

    #[serde(skip)]
    pub equipment: Vec<String>,
}

impl URLResource for AutoUncleVehicle {
    fn get_url(&self) -> String {
        format!("https://www.autouncle.ro/en/cars/{}", self.id)
    }
}

impl BasicT for AutoUncleVehicle {
    fn id(&self) -> String {
        self.id.clone()
    }
    fn source(&self) -> String {
        self.source.clone()
    }
    fn price(&self) -> Option<u32> {
        self.price
    }
    fn currency(&self) -> Currency {
        self.currency.unwrap_or(Currency::EUR)
    }
    fn make(&self) -> String {
        self.brand.clone().unwrap_or("".to_string())
    }
    fn model(&self) -> String {
        self.car_model.clone().unwrap_or("".to_string())
    }
    fn year(&self) -> u16 {
        self.year.unwrap_or(0)
    }
    fn power_ps(&self) -> u32 {
        self.hp.unwrap_or(0) as u32
    }
    fn gearbox(&self) -> Gearbox {
        if let Some(is_automatic) = self.has_auto_gear {
            if is_automatic {
                Gearbox::Automatic
            } else {
                Gearbox::Manual
            }
        } else {
            Gearbox::Manual
        }
    }
    fn engine(&self) -> Engine {
        if let Some(engine_fuel) = self.fuel.clone() {
            Engine::from_str(&engine_fuel).unwrap()
        } else {
            Engine::NotAvailable
        }
    }
    fn millage(&self) -> Option<u32> {
        self.km
    }
    fn cc(&self) -> u32 {
        (self.engine_size.unwrap_or(0.0) * 1000.0) as u32
    }
    fn power_kw(&self) -> u32 {
        self.kw.unwrap_or(0) as u32
    }
    fn month(&self) -> Option<u16> {
        None
    }
    fn title(&self) -> String {
        self.headline.clone().unwrap_or("".to_string())
    }
}

impl DetailsT for AutoUncleVehicle {
    fn get_id(&self) -> String {
        self.id.clone()
    }
    fn source(&self) -> String {
        self.source.clone()
    }
    fn phone(&self) -> String {
        "".to_string()
    }
    fn location(&self) -> String {
        self.location.clone()
    }
    fn equipment(&self) -> String {
        self.equipment.join(",")
    }

    fn is_dealer(&self) -> bool {
        self.seller_kind.to_lowercase() == "dealer"
    }
    fn view_count(&self) -> u32 {
        0
    }
    fn fuel_consumption(&self) -> f64 {
        self.fuel_economy.unwrap_or(0.0)
    }
    fn electric_drive_range(&self) -> f64 {
        self.electric_drive_range.unwrap_or(0.0)
    }

    fn cc(&self) -> u32 {
        (self.engine_size.unwrap_or(0.0) * 1000.0) as u32
    }

    fn seller_name(&self) -> String {
        self.source_name.clone()
    }
}

impl PriceT for AutoUncleVehicle {
    fn currency(&self) -> Currency {
        self.currency.unwrap_or(Currency::EUR)
    }
    fn price(&self) -> u32 {
        self.price.unwrap_or(0)
    }
    fn ranges(&self) -> Option<String> {
        None
    }

    fn overpriced_difference(&self) -> u32 {
        self.you_save_difference.unwrap_or(0)
    }

    fn save_difference(&self) -> u32 {
        self.you_save_difference.unwrap_or(0)
    }
    fn thresholds(&self) -> Vec<u32> {
        vec![]
    }
    fn estimated_price(&self) -> Option<u32> {
        self.estimated_price
    }
    fn id(&self) -> String {
        self.id.clone()
    }
    fn source(&self) -> String {
        self.source.clone()
    }
    fn rating(&self) -> Option<String> {
        None
    }
}

impl ChangeLogT for AutoUncleVehicle {
    fn get_id(&self) -> String {
        self.id.clone()
    }
    fn source(&self) -> String {
        self.source.clone()
    }
    fn published_on(&self) -> String {
        self.created_at.clone().unwrap_or("".to_string())
    }
    fn last_modified_on(&self) -> String {
        self.updated_at.clone().unwrap_or("".to_string())
    }
    fn last_modified_message(&self) -> String {
        "".to_string()
    }
    fn days_in_sale(&self) -> Option<u32> {
        self.laytime
    }
    fn sold(&self) -> bool {
        false
    }
    fn promoted(&self) -> bool {
        self.is_featured.unwrap_or(false)
    }
}
