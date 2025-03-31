use std::str::FromStr;

use log::info;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{ok_or_default, ok_or_err, unwrap_or_empty, unwrap_or_err};

use super::{
    DataConversionError::ConversionError,
    VehicleDataModel,
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
    type Error = ConversionError;

    fn try_from(item: SearchItem) -> Result<Self, Self::Error> {
        let id = unwrap_or_err!(item.id, "id");
        let mut base_info =
            VehicleDataModel::BaseVehicleInfo::new(id.to_string(), "mobile.de".to_string());

        if let Some(data) = &item.details {
            // Extract power

            // Extract consumption values
            let engine = ok_or_err!(Engine::from_str(unwrap_or_empty!(data.fuel_type)), "engine");

            // Extract gearbox
            let gearbox = ok_or_err!(Gearbox::from_str(unwrap_or_empty!(data.gearbox)), "gearbox");

            // Extract year
            let year = ok_or_err!(extract_year(unwrap_or_empty!(data.year)), "year");

            // Required fields
            let model = unwrap_or_err!(item.model.clone(), "model");
            let make = unwrap_or_err!(item.make.clone(), "make");
            let subtitle = unwrap_or_err!(item.title.clone(), "title");
            let (power_kw, power_ps) =
                ok_or_err!(extract_power(unwrap_or_empty!(data.power)), "power");
            let mileage = ok_or_default!(extract_ccm(unwrap_or_empty!(data.mileage)));
            let cc = ok_or_default!(extract_ccm(unwrap_or_empty!(data.cubic_capacity)));
            info!("Mileage: {} km", mileage);
            // Assign to base_info
            base_info.engine = engine;
            base_info.gearbox = gearbox;
            base_info.mileage = Some(mileage);
            base_info.model = model;
            base_info.make = make;
            base_info.power_kw = power_kw;
            base_info.power_ps = power_ps;
            base_info.year = year as u16;
            base_info.title = subtitle;
            base_info.cc = cc;
        }

        // Extract price safely
        base_info.price = Some(unwrap_or_err!(item.price, "price").gross_amount as u32);

        base_info.currency = Currency::EUR;

        base_info.url = format!(
            "https://suchen.mobile.de/fahrzeuge/details.html?id={}&lang=de&utm_source=DirectMail&utm_medium=textlink&utm_campaign=Recommend_DES&vc=Car",
            id
        );
        info!("URL: {:?}", &base_info);
        Ok(base_info)
    }
}

impl TryFrom<SearchItem> for VehicleDataModel::DetailedVehicleInfo {
    type Error = ConversionError;

    fn try_from(item: SearchItem) -> Result<Self, Self::Error> {
        if let Some(id) = item.id {
            let mut details =
                VehicleDataModel::DetailedVehicleInfo::new(id.to_string(), "mobile.de".to_string());
            details.source = "mobile.de".to_string();

            let contact = unwrap_or_err!(item.contactInfo, "contactInfo");
            details.location = contact.location;
            details.seller_name = contact.name.unwrap_or("".to_string());

            if let Some(data) = item.details {
                if let Ok((consumption_kw, consumption_fuel)) = extract_consumption(
                    data.consumption.as_ref().unwrap_or(&String::new()).as_str(),
                ) {
                    details.consumption_kw = consumption_kw;
                    details.consumption_fuel = consumption_fuel;
                }
            }
            Ok(details)
        } else {
            Err(ConversionError::MissingField("id".into()))
        }
    }
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

#[cfg(test)]
mod test_mobile_de {

    use log::{error, info};

    use super::{extract_ccm, extract_consumption, extract_integer, extract_power};

    // Extracts the first integer from a string

    // Extracts power values (kW and PS) from a string

    // Extracts consumption values (kWh/100km and l/100km)

    #[cfg(test)]
    mod tests {
        use std::{fs::File, io::Read};

        use crate::{
            LOG_CONFIG,
            model::{
                MobileDe::MobileDeResults,
                VehicleDataModel::{self, BaseVehicleInfo},
            },
            utils::helpers::configure_log4rs,
        };

        use super::*;

        #[test]
        fn test_extract_car_attributes() {
            configure_log4rs(&LOG_CONFIG);
            let mut file = File::open("mob.json").unwrap();
            let mut contents = String::new();
            let _ = file.read_to_string(&mut contents);

            let data: MobileDeResults = serde_json::from_str(&contents).unwrap();
            let mut counter = 0;
            data.search
                .srp
                .data
                .search_result
                .items
                .iter()
                .for_each(|item| {
                    if let Ok(result) = BaseVehicleInfo::try_from(item.clone()) {
                        info!("Basic Info {:?}", result);
                        counter += 1;
                    } else {
                        error!("Error: {:?}", item);
                    }

                    if let Ok(result) =
                        VehicleDataModel::DetailedVehicleInfo::try_from(item.clone())
                    {
                        info!("Detailed Info {:?}", result);
                    } else {
                        error!("Error");
                    }

                    if let Ok(price) = VehicleDataModel::Price::try_from(item.clone()) {
                        info!("Price Info {:?}", price);
                    } else {
                        error!("Error");
                    }
                });
            info!("Total: {}", counter);
        }

        #[test]
        fn test_extract_integer() {
            assert_eq!(extract_integer("26 g CO₂/km").unwrap(), 26);
            assert_eq!(extract_ccm("2,894 ccm").unwrap(), 2894);
            assert_eq!(extract_integer("0 km").unwrap(), 0);
            assert_eq!(extract_integer("").unwrap(), 0);
        }

        #[test]
        fn test_extract_power() {
            assert_eq!(extract_power("400 kW (544 hp)").unwrap(), (400, 544));
            assert_eq!(extract_power("250 kW (340 hp)").unwrap(), (250, 340));
            assert_eq!(extract_power("").unwrap(), (0, 0));
        }

        #[test]
        fn test_extract_consumption() {
            assert_eq!(
                extract_consumption("26.8 kWh\u{002F10}0km + 1.2 l\u{002F10}0km (wgt. comb.), 9.5 l\u{002F10}0km (discharged, comb.)").unwrap(),
                (26.8, 1.2)
            );
            assert_eq!(
                extract_consumption("15.3 kWh/100km + 5.5 l/100km").unwrap(),
                (15.3, 5.5)
            );
            assert_eq!(extract_consumption("").unwrap(), (0.0, 0.0));
        }
    }
}
