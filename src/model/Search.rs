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
}

impl From<HashMap<String, String>> for Search {
    fn from(params: HashMap<String, String>) -> Self {
        let raw_url = params.get("url").expect("missing url param");
        let id = params.get(ID_KEY).unwrap().to_string();
        let source = params.get(CRAWLER_KEY).unwrap().to_string();
        let final_url = if raw_url.starts_with("https://www.mobile.bg") {
            mobile_bg_url(raw_url, params.clone())
        } else {
            search_url(raw_url, params.clone())
        };
        let mut hasher = DefaultHasher::new();
        final_url.hash(&mut hasher);
        let hash = hasher.finish().to_string();
        Search {
            id,
            url: final_url,
            hash,
            source,
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
    // If the URL has no placeholders, return it as-is
    if !url.contains('{') {
        return url.to_string();
    }

    // Work on a mutable copy
    let mut out = url.to_string();

    // Replace price placeholders if present
    if out.contains("{priceFrom}") {
        if let Some(from) = params.get("priceFrom") {
            out = out.replace("{priceFrom}", &format!("price={}", from));
        } else {
            out = out.replace("{priceFrom}", "");
        }
    }
    if out.contains("{priceTo}") {
        if let Some(to) = params.get("priceTo") {
            out = out.replace("{priceTo}", &format!("&price1={}", to));
        } else {
            out = out.replace("{priceTo}", "");
        }
    }

    // Replace year placeholders if present
    if out.contains("{yearFrom}") {
        if let Some(from_year) = params.get(MOBILE_BG_YEARS_FROM) {
            out = out.replace("{yearFrom}", from_year);
        } else {
            out = out.replace("{yearFrom}", "");
        }
    }
    if out.contains("{yearTo}") {
        if let Some(to_year) = params.get(MOBILE_BG_YEARS_TO) {
            out = out.replace("{yearTo}", to_year);
        } else {
            out = out.replace("{yearTo}", "");
        }
    }

    out
}
