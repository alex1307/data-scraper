use std::str::FromStr;

use log::info;
use serde::{Deserialize, Serialize};

use super::{
    enums::{Currency, Engine, Gearbox},
    VehicleDataModel,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MobileDeResults {
    #[serde(rename = "search")]
    pub search: Search,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Search {
    pub srp: SRP,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SRP {
    #[serde(rename = "data")]
    pub data: SRPData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SRPData {
    #[serde(rename = "metaData")]
    pub meta_data: MetaData,

    #[serde(rename = "aggregations")]
    pub aggregations: Aggregations,

    #[serde(rename = "searchResults")]
    pub search_result: SearchResult,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetaData {
    pub(crate) title: String,
    pub(crate) headline: String,
    pub(crate) description: String,
    pub(crate) keywords: String,
    pub(crate) breadcrumbs: Vec<Breadcrumb>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Aggregations {
    pub(crate) st: Vec<KeyCount>,
    pub(crate) clim: Vec<KeyCount>,
    pub(crate) fe: Vec<KeyCount>,
    // Assuming `dm` is similar in structure to `st`, `clim`, etc.
    pub(crate) dm: Vec<KeyCount>,
    pub(crate) sr: Vec<KeyCount>,
    // Add other fields if needed
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyCount {
    pub(crate) key: String,
    pub(crate) count: u64,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Breadcrumb {
    pub(crate) label: String,
    pub(crate) href: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResult {
    #[serde(rename = "searchId")]
    pub search_id: String,

    #[serde(rename = "numResultsTotal")]
    pub total_results: u32,

    #[serde(rename = "page")]
    pub page: u32,

    #[serde(rename = "numPages")]
    pub number_of_pages: u32,

    #[serde(rename = "obsSearchResultsCount")]
    obs_search_results_count: u32,

    #[serde(rename = "hasNextPage")]
    pub has_next_page: bool,

    #[serde(rename = "items")]
    pub items: Vec<SearchItem>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchItem {
    pub isEyeCatcher: Option<bool>,
    pub searchId: Option<String>,
    pub hasElectricEngine: Option<bool>,
    //financePlans: Vec<FinancePlan>,
    // sellerId: u64,
    pub priceRating: Option<PriceRating>,
    pub segment: Option<String>,
    pub title: Option<String>,
    pub vc: Option<String>,
    pub category: Option<String>,
    pub id: Option<u64>,
    // customDimensions: std::collections::HashMap<String, String>,
    // obsUrl: String,
    // relativeUrl: String,
    pub attributes: Option<Vec<String>>,
    pub contactInfo: Option<ContactInfo>,
    // //    previewImage: Image,
    // //    previewThumbnails: Vec<Image>,
    pub price: Option<Price>,
    // isFinancingAvailable: bool,
    pub make: Option<String>,
    pub model: Option<String>,
    #[serde(rename = "type")]
    pub modelType: Option<String>,
    // emailLink: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FinancePlan {
    #[serde(rename = "type")]
    plan_type: String,
    url: String,
    showInGallery: bool,
    offer: FinanceOffer,
    budgetStatus: String,
    fallback: bool,
    downPayment: u32,
    loanDuration: u32,
    localized: FinanceLocalized,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FinanceOffer {
    bankName: String,
    loanBroker: String,
    minMonthlyInstallment: u32,
    minInterestRateEffective: f64,
    minInterestRateNominal: f64,
    maxMonthlyInstallment: u32,
    maxInterestRateEffective: f64,
    maxInterestRateNominal: f64,
    localized: FinanceLocalized,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FinanceLocalized {
    minMonthlyInstallment: String,
    minInterestRateEffective: String,
    minInterestRateNominal: String,
    maxMonthlyInstallment: String,
    maxInterestRateEffective: String,
    maxInterestRateNominal: String,
    disclaimer: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PriceRating {
    pub(crate) rating: String,
    pub(crate) ratingLabel: String,
    pub(crate) thresholdLabels: Option<Vec<String>>,
    pub(crate) vehiclePriceOffset: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Kba {
    hsn: String,
    tsn: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContactInfo {
    pub typeLocalized: String,
    pub name: Option<String>,
    pub location: String,
    pub rating: Option<Rating>,
    pub hasContactPhones: bool,
    pub contactPhone: Option<String>,
    pub country: String,
    pub sellerType: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rating {
    pub score: f64,
    pub count: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Image {
    src: String,
    srcSet: String,
    alt: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Price {
    pub(crate) gross: String,

    #[serde(rename = "grossAmount")]
    pub(crate) gross_amount: f64,

    #[serde(rename = "grossCurrency")]
    pub(crate) currency: String,

    pub(crate) net: Option<String>,

    #[serde(rename = "netAmount")]
    pub(crate) net_amount: Option<f64>,

    pub(crate) vat: Option<String>,
}

impl TryFrom<SearchItem> for VehicleDataModel::Price {
    type Error = String;

    fn try_from(item: SearchItem) -> Result<Self, Self::Error> {
        if let Some(id) = item.id {
            let mut price = VehicleDataModel::Price::new(id.to_string(), "mobile.de".to_string());
            if let Some(itemPrice) = item.price {
                price.price = itemPrice.gross_amount as u32;
                price.currency = Currency::EUR;
            }
            if let Some(rating) = item.priceRating {
                if let Some(threshold_labels) = rating.thresholdLabels {
                    let mut thresholds = vec![];
                    for s in &threshold_labels {
                        let number = s.chars().filter(|c| c.is_ascii_digit()).collect::<String>();
                        match number.parse::<u32>() {
                            Ok(number) => thresholds.push(number),
                            Err(e) => {
                                info!("Error: {:?}", e);
                                continue;
                            }
                        }
                    }
                    price.thresholds = thresholds.clone();

                    if thresholds.len() == 6 {
                        let p1 = (thresholds[0] + thresholds[5]) / 2;
                        let p2 = (thresholds[1] + thresholds[4]) / 2;
                        let p3 = (thresholds[2] + thresholds[3]) / 2;
                        price.estimated_price = Some((p1 + p2 + p3) / 3);
                        price.ranges = Some(format!(
                            "[{},{},{},{},{},{}]",
                            thresholds[0],
                            thresholds[1],
                            thresholds[2],
                            thresholds[3],
                            thresholds[4],
                            thresholds[5]
                        ));
                    } else {
                        price.estimated_price =
                            Some((thresholds[0] + thresholds[thresholds.len() - 1]) / 2);
                        price.ranges = Some(
                            thresholds
                                .iter()
                                .map(|t| t.to_string())
                                .collect::<Vec<String>>()
                                .join(",")
                                .to_string(),
                        );
                    }
                    if price.price < price.estimated_price.unwrap() {
                        price.save_difference = price.estimated_price.unwrap() - price.price;
                    } else {
                        price.overpriced_difference = price.price - price.estimated_price.unwrap();
                    }
                }
                price.rating = Some(rating.rating);
            }
            Ok(price)
        } else {
            Err("No id found".into())
        }
    }
}

impl TryFrom<SearchItem> for VehicleDataModel::BaseVehicleInfo {
    type Error = String;

    fn try_from(item: SearchItem) -> Result<Self, Self::Error> {
        if let Some(id) = item.id {
            let mut base_info = VehicleDataModel::BaseVehicleInfo::new(id.to_string());
            base_info.source = "mobile.de".to_string();
            if let Some(attributes) = item.attributes {
                info!("attributes: {:?}", attributes);
                let flattened_attributes: Vec<String> = attributes
                    .iter()
                    .flat_map(|a| a.split(" • "))
                    .map(|s| s.to_string())
                    .collect();

                let milage = match flattened_attributes[1]
                    .chars()
                    .filter(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse::<u32>()
                {
                    Ok(milage) => milage,
                    Err(e) => {
                        info!("Error: {:?}", e);
                        0
                    }
                };

                base_info.millage = Some(milage);
                base_info.year = get_year(&flattened_attributes[0]);

                if flattened_attributes[2].contains("kW") {
                    let kw_ps = flattened_attributes[2].split("kW").collect::<Vec<&str>>();
                    let mut power = vec![];
                    for s in kw_ps {
                        let number = match s
                            .chars()
                            .filter(|c| c.is_ascii_digit())
                            .collect::<String>()
                            .parse::<u32>()
                        {
                            Ok(number) => number,
                            Err(e) => {
                                info!("Error: {:?}", e);
                                0
                            }
                        };

                        power.push(number);
                    }
                    if power.len() == 2 {
                        base_info.power_kw = power[0];
                        base_info.power_ps = power[1];
                    } else {
                        base_info.power_kw = power[0];
                    }
                }

                let mut engine: Engine = Engine::NotAvailable;
                let mut gearbox: Gearbox = Gearbox::NotAvailable;
                for attr in &flattened_attributes {
                    engine = Engine::from_str(attr.as_str()).unwrap();
                    if Engine::NotAvailable == engine {
                        continue;
                    } else {
                        break;
                    }
                }

                for attr in &flattened_attributes {
                    gearbox = Gearbox::from_str(attr.as_str()).unwrap();
                    if Gearbox::NotAvailable == gearbox {
                        continue;
                    } else {
                        break;
                    }
                }
                base_info.engine = engine;
                base_info.gearbox = gearbox;
            }
            if let Some(model) = item.model {
                base_info.model = model;
            }
            if let Some(make) = item.make {
                base_info.make = make;
            }
            if let Some(title) = item.title {
                base_info.title = title;
            }

            if let Some(itemPrice) = item.price {
                base_info.price = Some(itemPrice.gross_amount as u32);
                base_info.currency = Currency::EUR;
            }

            Ok(base_info)
        } else {
            Err("No id found".into())
        }
    }
}

impl TryFrom<SearchItem> for VehicleDataModel::Consumption {
    type Error = String;

    fn try_from(item: SearchItem) -> Result<Self, Self::Error> {
        if let Some(id) = item.id {
            let mut consumption =
                VehicleDataModel::Consumption::new(id.to_string(), "mobile.de".to_string());
            if let Some(make) = item.make {
                consumption.make = make;
            }

            if let Some(model) = item.model {
                consumption.model = model;
            }

            if let Some(attrbutes) = item.attributes {
                let flattened_attributes: Vec<String> = attrbutes
                    .iter()
                    .flat_map(|a| a.split(" • "))
                    .map(|s| s.to_string())
                    .collect();
                consumption.year = get_year(&flattened_attributes[0]);

                for attr in flattened_attributes {
                    if attr.contains("kWh/100km") {
                        let kWh = attr.split("kWh/100km").collect::<Vec<&str>>()[0].trim();
                        if kWh.contains('.') {
                            consumption.kw_consuption = match kWh.parse::<f32>() {
                                Ok(kw) => Some(kw),
                                Err(e) => {
                                    info!("Error: {:?}", e);
                                    None
                                }
                            };
                        } else if kWh.contains(',') {
                            consumption.kw_consuption = match kWh.replace(',', ".").parse::<f32>() {
                                Ok(kw) => Some(kw),
                                Err(e) => {
                                    info!("Error: {:?}", e);
                                    None
                                }
                            };
                        }
                    } else if attr.contains("l/100km") {
                        let l = attr.split("l/100km").collect::<Vec<&str>>()[0].trim();
                        let l = l.replace("ca.", "ca:");
                        let l = l
                            .chars()
                            .filter(|c| c.is_ascii_digit() || *c == ',' || *c == '.')
                            .collect::<String>();
                        if l.contains('.') {
                            consumption.fuel_consumption = match l.parse::<f32>() {
                                Ok(fuel) => Some(fuel),
                                Err(e) => {
                                    info!("Error: {:?}", e);
                                    None
                                }
                            };
                        } else if l.contains(',') {
                            consumption.fuel_consumption = match l.replace(',', ".").parse::<f32>()
                            {
                                Ok(fuel) => Some(fuel),
                                Err(e) => {
                                    info!("Error: {:?}", e);
                                    None
                                }
                            };
                        }
                    } else if attr.contains("CO₂/km") {
                        consumption.co2_emission = match attr
                            .chars()
                            .filter(|c| c.is_ascii_digit())
                            .collect::<String>()
                            .parse::<u32>()
                        {
                            Ok(co2) => co2,
                            Err(e) => {
                                info!("Error: {:?}", e);
                                0
                            }
                        };
                    }
                }
            }
            Ok(consumption)
        } else {
            Err("No id found".into())
        }
    }
}

fn get_year(year: &str) -> u16 {
    if "new car" == year.trim().to_lowercase() || "neuwagen" == year.trim().to_lowercase() {
        return 2024;
    }

    if year.contains('/') {
        let month_year = year.split(' ').collect::<Vec<&str>>();
        if month_year.len() > 1 {
            let prod_year = month_year[1].split('/').collect::<Vec<&str>>();
            if prod_year.len() == 1 {
                return match prod_year[0].parse::<u16>() {
                    Ok(year) => year,
                    Err(e) => {
                        info!("Error: {:?}", e);
                        0
                    }
                };
            } else {
                return match prod_year[1].parse::<u16>() {
                    Ok(year) => year,
                    Err(e) => {
                        info!("Error: {:?}", e);
                        0
                    }
                };
            }
        }
    }

    0
}
