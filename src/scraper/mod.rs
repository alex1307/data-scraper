use std::collections::HashMap;

use crate::services::SearchBuilder::EXCLUED;

pub mod AutouncleFRScraper;
pub mod AutouncleNLScraper;
pub mod AutouncleROScraper;
pub mod CarsBgScraper;
pub mod MobileBgScraper;
pub mod MobileDeFileScraper;
pub mod Traits;

pub fn search_url(url: String, path: Option<String>, params: HashMap<String, String>) -> String {
    let mut url = if let Some(path) = path {
        format!("{}{}", url, path)
    } else {
        url.clone()
    };

    if params.is_empty() {
        return url;
    }

    let mut keys = params
        .keys()
        .map(|k| k.to_string())
        .collect::<Vec<String>>();
    keys.sort_by_key(|k| k.to_lowercase());

    for key in keys {
        if EXCLUED.contains(&key.as_str()) {
            continue;
        }
        let value = params.get(&key).unwrap();
        if value.contains('[') && value.contains(']') {
            let value = value.replace(['[', ']'], "");
            let values: Vec<&str> = value.split(',').collect();
            for value in values {
                url = format!("{}{}={}&", url, key, value);
            }
            continue;
        }
        url = format!("{}{}={}&", url, key, value);
    }

    url
}
