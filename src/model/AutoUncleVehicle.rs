use serde::{Deserialize, Serialize};

use super::enums::Currency;

#[derive(Serialize, Deserialize, Debug)]
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
    pub laytime: u8,

    #[serde(rename = "sellerKind")]
    pub seller_kind: String,

    #[serde(rename = "isElectric")]
    pub is_electric: bool,

    #[serde(rename = "priceChange")]
    pub price_change: Option<i32>,
}
