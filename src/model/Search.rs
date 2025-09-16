use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};
use url::form_urlencoded;

use crate::services::SearchBuilder::{CRAWLER_KEY, ID_KEY};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
lazy_static! {
    pub static ref EXCLUED: Vec<&'static str> = vec![
        "seller",
        "engine",
        "gearbox",
        "power",
        ID_KEY,
        CRAWLER_KEY,
        "url"
    ];
}

pub const MOBILE_BG_POWER_FROM: &str = "powerFrom";
pub const MOBILE_BG_POWER_TO: &str = "powerTo";
pub const MOBILE_BG_YEARS_FROM: &str = "yearFrom";
pub const MOBILE_BG_YEARS_TO: &str = "yearTo";
pub const SE_BG_SELLER_TO: &str = "dealer";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Search {
    pub id: String,
    pub url: String,
    pub source: String,
    pub hash: String,
    pub engine: Option<String>,
    pub gearbox: Option<String>,
    pub power: Option<String>,
}

impl From<HashMap<String, String>> for Search {
    fn from(params: HashMap<String, String>) -> Self {
        let url = params.get("url").unwrap();
        let id = params.get(ID_KEY).unwrap().to_string();
        let source = params.get(CRAWLER_KEY).unwrap().to_string();
        let engine = params.get("engine").map(|x| x.to_string());
        let gearbox = params.get("gearbox").map(|x| x.to_string());
        let power = params.get("power").map(|x| x.to_string());
        let url = if url.starts_with("https://www.mobile.bg") {
            mobile_bg_url(url, params.clone())
        } else {
            search_url(url, params.clone())
        };
        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        let hash = hasher.finish().to_string();
        Search {
            id,
            url,
            hash,
            source,
            engine,
            gearbox,
            power,
        }
    }
}

fn search_url(base: &str, params: HashMap<String, String>) -> String {
    if params.is_empty() {
        return base.to_string();
    }

    let mut serializer = form_urlencoded::Serializer::new(String::new());

    let mut keys: Vec<String> = params.keys().cloned().collect();
    keys.sort_by_key(|k| k.to_lowercase());

    for key in keys {
        if EXCLUED.contains(&key.as_str()) {
            continue;
        }
        let value = params.get(&key).unwrap();
        if value.contains(',') {
            for v in value.split(',') {
                serializer.append_pair(&key, v);
            }
        } else {
            serializer.append_pair(&key, value);
        }
    }

    let query = serializer.finish();
    format!("{}{}", base, query)
}

fn mobile_bg_url(url: &str, params: HashMap<String, String>) -> String {
    let url = if let Some(from) = params.get("priceFrom") {
        url.replace("{priceFrom}", format!("price={}", from).as_str())
    } else {
        url.replace("{priceFrom}", "")
    };
    let url = if let Some(to) = params.get("priceTo") {
        url.replace("{priceTo}", format!("&price1={}", to).as_str())
    } else {
        url.replace("{priceTo}", "")
    };

    let fromYear = params.get(MOBILE_BG_YEARS_FROM).unwrap();
    let toYear = params.get(MOBILE_BG_YEARS_TO).unwrap();

    let url = url.replace("{yearFrom}", fromYear);
    url.replace("{yearTo}", toYear)
}
