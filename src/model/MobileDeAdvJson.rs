use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{ok_or_message, unwrap_or_message};

use super::{DataConversionError::ConversionError, MobileDe::SearchItem};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)] // Allows handling different structures (VehicleData vs NestedItem)
pub enum Item {
    Vehicle(SearchItem), // A regular vehicle entry
    Nested(NestedItem),
    Unknown(UnknownItem), // A nested item containing more items
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NestedItem {
    items: Vec<Item>, // Recursively contains more items
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnknownItem;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MobileJsonDeResults {
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
    pub items: Vec<Item>,
}
const SEARCH_ITEM: &str = "/search/srp/data/searchResults/items";
const NESTED_ITEMS: &str = "/items";
pub fn processMobileDeJson(contents: &str) -> Result<Vec<SearchItem>, ConversionError> {
    let mut allitems = vec![];
    let json: Value = ok_or_message!(
        serde_json::from_str(contents),
        "Failed to parse JSON".to_string()
    );
    let items = unwrap_or_message!(
        json.pointer(SEARCH_ITEM),
        "No 'items' found in JSON. Structure might be incorrect.".to_string()
    );
    let items_array = unwrap_or_message!(
        items.as_array(),
        "Expected items to be an array".to_string()
    );
    for item in items_array.iter() {
        if item.pointer(NESTED_ITEMS).is_some() {
            let items = unwrap_or_message!(
                item.pointer(NESTED_ITEMS),
                "Expected items to be an array".to_string()
            );
            let item_arr: Vec<Item> = ok_or_message!(
                serde_json::from_value(items.clone()),
                "Failed to parse items".to_string()
            );
            allitems.extend(item_arr.into_iter().filter_map(|i| match i {
                Item::Vehicle(v) => Some(v),
                _ => None,
            }));
        } else {
            let item: Item = ok_or_message!(
                serde_json::from_value(item.clone()),
                "Failed to parse item".to_string()
            );
            if let Item::Vehicle(v) = item {
                allitems.push(v);
            }
        }
    }

    Ok(allitems)
}

#[cfg(test)]
mod mobile_de_json_test {
    use std::{fs::File, io::Read};

    use log::{error, info};

    use crate::{
        model::VehicleDataModel::{BaseVehicleInfo, DetailedVehicleInfo, Price},
        utils::helpers::configure_log4rs,
        LOG_CONFIG,
    };

    #[test]
    fn test_extract_car_attributes() {
        configure_log4rs(&LOG_CONFIG);
        let mut file = File::open("mob.json").unwrap();
        let mut contents = String::new();
        let _ = file.read_to_string(&mut contents);

        let searchItems = super::processMobileDeJson(&contents).unwrap();

        let mut allbase = vec![];
        let mut alldetailed = vec![];
        let mut allprice = vec![];
        info!("All items: {:?}", searchItems.len());
        assert_eq!(30, searchItems.len());
        for item in searchItems.iter() {
            match BaseVehicleInfo::try_from(item.clone()) {
                Ok(base) => {
                    allbase.push(base);
                }
                Err(e) => {
                    error!(
                        "❌ Failed to convert to BaseVehicleInfo: {:?}, Error: {:?}",
                        item, e
                    );
                }
            }
            match DetailedVehicleInfo::try_from(item.clone()) {
                Ok(detailed) => {
                    alldetailed.push(detailed);
                }
                Err(e) => {
                    error!(
                        "❌ Failed to convert to DetailedVehicleInfo: {:?}, Error: {:?}",
                        item, e
                    );
                }
            }

            match Price::try_from(item.clone()) {
                Ok(price) => {
                    allprice.push(price);
                }
                Err(e) => {
                    error!("❌ Failed to convert to Price: {:?}, Error: {:?}", item, e);
                }
            }
        }
        assert_eq!(24, allbase.len());
        assert_eq!(24, alldetailed.len());
        assert_eq!(24, allprice.len());
    }
}
