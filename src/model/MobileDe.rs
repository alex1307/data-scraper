use std::str::FromStr;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{ok_or_default, ok_or_err, unwrap_or_empty, unwrap_or_err};

use super::{
    DataConversionError::ConversionError,
    VehicleDataModel::Vehicle,
    enums::{Currency, Engine, Gearbox},
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
    // /Assuming `dm` is similar in structure to `st`, `clim`, etc.
    pub(crate) dm: Vec<KeyCount>,
    pub(crate) sr: Vec<KeyCount>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyCount {
    //pub(crate) key: String,
    pub(crate) count: u64,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Breadcrumb {
    pub(crate) label: String,
    //    pub(crate) href: Option<String>,
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
    //pub segment: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "subTitle")]
    pub subTitle: Option<String>,
    //pub vc: Option<String>,
    //pub category: Option<String>,
    pub id: Option<u64>,
    // customDimensions: std::collections::HashMap<String, String>,
    // obsUrl: String,
    // relativeUrl: String,
    attributes: Option<Vec<Vec<Attribute>>>,
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
    #[serde(rename = "attr")]
    pub details: Option<VehicleDetails>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VehicleDetails {
    #[serde(rename = "cn")]
    pub country: Option<String>,
    #[serde(rename = "fr")]
    pub year: Option<String>,
    #[serde(rename = "z")]
    pub zip: Option<String>,
    #[serde(rename = "loc")]
    pub location: Option<String>,
    #[serde(rename = "pw")]
    pub power: Option<String>,
    #[serde(rename = "ft")]
    pub fuel_type: Option<String>,
    #[serde(rename = "ml")]
    pub mileage: Option<String>,
    #[serde(rename = "cc")]
    pub cubic_capacity: Option<String>,
    #[serde(rename = "csmpt")]
    pub consumption: Option<String>,
    #[serde(rename = "emiss")]
    pub emission: Option<String>,
    #[serde(rename = "co2class")]
    pub co2: Option<String>,
    #[serde(rename = "tr")]
    pub gearbox: Option<String>,
    #[serde(rename = "con")]
    pub condition: Option<String>,
    #[serde(rename = "ecol")]
    pub color: Option<String>,
    #[serde(rename = "eu")]
    pub edition: Option<String>,
    #[serde(rename = "door")]
    pub doors: Option<String>,
    #[serde(rename = "c")]
    pub category: Option<String>,
    #[serde(rename = "pvo")]
    pub previousOwners: Option<String>,
    #[serde(rename = "nw")]
    pub weight: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Attribute {
    value: String,
    bold: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FinancePlan {
    #[serde(rename = "type")]
    plan_type: String,
    url: String,
    showInGallery: bool,
    //offer: FinanceOffer,
    budgetStatus: String,
    fallback: bool,
    downPayment: u32,
    loanDuration: u32,
    //localized: FinanceLocalized,
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
    //    pub(crate) ratingLabel: String,
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

fn extract_year(input: &str) -> Result<u16, Box<dyn std::error::Error>> {
    let re = Regex::new(r"\b(\d{4})\b")?; // Match a 4-digit year

    if let Some(caps) = re.captures(input) {
        let year = caps.get(1).unwrap().as_str().parse::<u16>()?;
        return Ok(year);
    }

    Ok(0) // Default to 0 if no year is found
}

fn extract_integer(input: &str) -> Result<u32, Box<dyn std::error::Error>> {
    let re = Regex::new(r"\d+")?;
    if let Some(mat) = re.find(input) {
        Ok(mat.as_str().parse::<u32>()?)
    } else {
        Ok(0)
    }
}

fn extract_ccm(input: &str) -> Result<u32, Box<dyn std::error::Error>> {
    let filtered = input.replace(",", "");
    extract_integer(&filtered)
}

// Extracts power values (kW and PS) from a string
fn extract_power(input: &str) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    let re = Regex::new(r"(\d+)\s*kW\s*\((\d+)\s*hp\)")?;
    if let Some(caps) = re.captures(input) {
        let kw = caps.get(1).unwrap().as_str().parse::<u32>()?;
        let ps = caps.get(2).unwrap().as_str().parse::<u32>()?;
        Ok((kw, ps))
    } else {
        Ok((0, 0))
    }
}

// Extracts consumption values (kWh/100km and l/100km)
fn extract_consumption(input: &str) -> Result<(f32, f32), Box<dyn std::error::Error>> {
    let re = Regex::new(r"([\d\.]+)\s*kWh.*?([\d\.]+)\s*l")?;
    if let Some(caps) = re.captures(input) {
        let kw = caps
            .get(1)
            .unwrap()
            .as_str()
            .replace(',', ".")
            .parse::<f32>()?;
        let fuel = caps
            .get(2)
            .unwrap()
            .as_str()
            .replace(',', ".")
            .parse::<f32>()?;
        Ok((kw, fuel))
    } else {
        Ok((0.0, 0.0))
    }
}

impl TryFrom<SearchItem> for Vehicle {
    type Error = ConversionError;

    fn try_from(item: SearchItem) -> Result<Self, Self::Error> {
        let id = unwrap_or_err!(item.id, "id").to_string();
        let make = unwrap_or_err!(item.make.clone(), "make");
        let model = unwrap_or_err!(item.model.clone(), "model");
        let title = unwrap_or_err!(item.title.clone(), "title");

        let details = unwrap_or_err!(item.details.clone(), "details");

        let engine = ok_or_err!(
            Engine::from_str(unwrap_or_empty!(details.fuel_type)),
            "engine"
        );
        let gearbox = ok_or_err!(
            Gearbox::from_str(unwrap_or_empty!(details.gearbox)),
            "gearbox"
        );
        let year = ok_or_err!(extract_year(unwrap_or_empty!(details.year)), "year");
        let (power_kw, power_ps) =
            ok_or_err!(extract_power(unwrap_or_empty!(details.power)), "power");

        let mileage = ok_or_default!(extract_ccm(unwrap_or_empty!(details.mileage)));
        let cc = ok_or_default!(extract_ccm(unwrap_or_empty!(details.cubic_capacity)));

        let price_obj = unwrap_or_err!(item.price.clone(), "price");
        let price = price_obj.gross_amount as u32;
        let currency = Currency::EUR;

        let url = format!(
            "https://suchen.mobile.de/fahrzeuge/details.html?id={}&lang=de&utm_source=DirectMail&utm_medium=textlink&utm_campaign=Recommend_DES&vc=Car",
            id
        );

        let contact = unwrap_or_err!(item.contactInfo.clone(), "contactInfo");
        let location = Some(contact.location.clone());
        let seller_name = contact.name;
        let seller_url = None; // Could be filled in if available from `item`

        let mut estimated_price = None;
        let mut thresholds = vec![];
        let mut ranges = None;
        let mut rating = None;

        if let Some(price_rating) = item.priceRating.clone() {
            rating = Some(price_rating.rating.clone());
            if let Some(labels) = price_rating.thresholdLabels {
                for label in &labels {
                    let digits: String = label.chars().filter(|c| c.is_ascii_digit()).collect();
                    if let Ok(val) = digits.parse::<u32>() {
                        thresholds.push(val);
                    }
                }

                if thresholds.len() >= 2 {
                    estimated_price = Some((thresholds[0] + thresholds[thresholds.len() - 1]) / 2);
                }

                ranges = Some(
                    thresholds
                        .iter()
                        .map(|t| t.to_string())
                        .collect::<Vec<_>>()
                        .join(","),
                );
            }
        }

        let (consumption_kw, consumption_fuel) = if let Some(ref c) = details.consumption {
            extract_consumption(c).unwrap_or((0.0, 0.0))
        } else {
            (0.0, 0.0)
        };

        let co2 = ok_or_default!(extract_integer(unwrap_or_empty!(details.co2)));

        Ok(Vehicle {
            id,
            source: "mobile.de".to_string(),
            make,
            model,
            title,
            year,
            mileage,
            engine,
            gearbox,
            power_ps,
            power_kw,
            currency,
            price,
            estimated_price,
            cc: Some(cc),
            url,
            location,
            equipment: None, // Not present in source, you can extend if available
            seller_name,
            seller_url,
            range: None, // Not available from SearchItem
            consumption_fuel: Some(consumption_fuel),
            consumption_kw: Some(consumption_kw),
            co2: Some(co2),
            days_in_sale: None, // Not available from SearchItem
            ranges,
            rating,
            thresholds,
        })
    }
}
