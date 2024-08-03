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
pub struct Root {
    pub dynamicScriptData: DynamicScriptData,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DynamicScriptData {
    pub cars_search: CarsSearch,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CarsSearch {
    pub carsPaginated: CarsPaginated,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CarsPaginated {
    pub cars: Vec<AutoUncleVehicle>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AutoUncleVehicle {
    announcedAsNew: Option<bool>,
    auRating: Option<u8>,
    availableForOnlineSales: Option<bool>,
    brand: Option<String>,
    body: Option<String>,
    carModel: Option<String>,
    co2Emission: Option<f64>,
    currency: Option<Currency>,
    displayableFuelConsumption: Option<String>, // Assuming type
    displayableImages: Vec<Image>,
    doors: Option<u8>,
    electricDriveRange: Option<f64>, // Assuming type
    engineSize: Option<f64>,
    equipmentVariant: Option<String>,
    estimatedPrice: Option<u32>,
    pub featuredAttributesEquipment: Vec<String>,
    pub featuredAttributesNonEquipment: Vec<String>,
    freeDelivery: Option<bool>,
    fuel: Option<String>,
    fuelEconomy: Option<f64>, // Assuming type
    hasAutoGear: Option<bool>,
    headline: Option<String>,
    highestPriorityActiveAdvert: HighestPriorityActiveAdvert,
    hp: Option<u32>,
    id: String,
    isElectric: Option<bool>,
    isFeatured: Option<bool>,
    km: Option<u32>,
    kw: Option<u32>,
    laytime: Option<u32>,
    localizedFuelEconomyLabel: Option<String>,
    location: String,
    mileageUnit: Option<String>,
    modelGeneration: Option<String>,
    noRatingReasons: Vec<String>, // Assuming this is correct
    pub outgoingPath: Option<String>,
    price: Option<u32>,
    priceChange: Option<f64>, // Assuming type
    regMonth: Option<String>,
    sellerKind: Option<String>,
    sourceName: Option<String>,
    vdpPath: Option<String>,
    year: Option<u16>,
    youSaveDifference: Option<u32>,
    created_at: Option<String>,
    updated_at: Option<String>,

    #[serde(skip)]
    pub source: String,

    #[serde(skip)]
    pub equipment: Vec<String>,

    #[serde(skip)]
    pub searchId: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Image {
    mediumUrl: String,
    smallUrl: String,
    url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct HighestPriorityActiveAdvert {
    isLocationIndependent: Option<bool>,
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
        self.carModel.clone().unwrap_or("".to_string())
    }
    fn year(&self) -> u16 {
        self.year.unwrap_or(0)
    }
    fn power_ps(&self) -> u32 {
        self.hp.unwrap_or(0) as u32
    }
    fn gearbox(&self) -> Gearbox {
        if let Some(is_automatic) = self.hasAutoGear {
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
        (self.engineSize.unwrap_or(0.0) * 1000.0) as u32
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
    fn search_id(&self) -> String {
        self.searchId.clone()
    }
    fn url(&self) -> String {
        if let Some(url) = &self.outgoingPath {
            match self.source.as_str() {
                "autouncle.ro" => return format!("https://www.autouncle.ro{}", url),
                "autouncle.fr" => return format!("https://www.autouncle.fr{}", url),
                "autouncle.nl" => return format!("https://www.autouncle.nl{}", url),
                "autouncle.ch" => return format!("https://www.autouncle.ch{}", url),
                "autouncle.pl" => return format!("https://www.autouncle.pl{}", url),
                _ => "".to_string(),
            }
        } else {
            "".to_string()
        }
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

    fn seller_name(&self) -> String {
        self.sourceName.clone().unwrap_or_default()
    }

    fn seller_url(&self) -> String {
        "".to_string()
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
        self.youSaveDifference.unwrap_or(0)
    }

    fn save_difference(&self) -> u32 {
        self.youSaveDifference.unwrap_or(0)
    }
    fn thresholds(&self) -> Vec<u32> {
        vec![]
    }
    fn estimated_price(&self) -> Option<u32> {
        self.estimatedPrice
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
        self.isFeatured.unwrap_or(false)
    }
}
