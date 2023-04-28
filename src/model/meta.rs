use log::info;
use serde::{Deserialize, Serialize};

use crate::{
    downloader::{
        Scraper::{get_header_data, get_pages},
        Utils::extract_ascii_latin,
    },
    listing_url,
};

use super::traits::{Header, Identity};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MetaHeader {
    pub timestamp: String,
    pub meta_type: String,
    pub make: String,
    pub model: String,
    pub total_number: u32,
    pub min_price: u32,
    pub max_price: u32,
    pub created_on: String,
    pub dealer: String,
}

impl Identity for MetaHeader {
    fn get_id(&self) -> String {
        self.timestamp.clone()
    }
}

impl Header for MetaHeader {
    fn header() -> Vec<&'static str> {
        vec![
            "timestamp",
            "dealer",
            "meta_type",
            "make",
            "model",
            "total_number",
            "min_price",
            "max_price",
            "created_on",
        ]
    }
}

impl MetaHeader {
    pub fn from_slink(slink: &str) -> Self {
        let url = &listing_url(slink, 1);
        let html = get_pages(url).unwrap();
        let content = get_header_data(&html).unwrap();
        let meta = extract_ascii_latin(&content);
        let re = regex::Regex::new(r" {2,}").unwrap();
        let split: Vec<&str> = re.split(meta.trim()).collect();
        let min_price = split[0].replace(' ', "").parse::<u32>().unwrap_or(0);
        let max_price = split[1].replace(' ', "").parse::<u32>().unwrap_or(0);
        let total_number = split[2].replace(' ', "").parse::<u32>().unwrap_or(0);
        if split.len() <= 4 {
            return MetaHeader {
                min_price,
                max_price,
                total_number,
                ..Default::default()
            };
        }

        let make_model: Vec<&str> = split[0].split_whitespace().collect();
        let (make, model) = if make_model.len() == 1 {
            (make_model[0], "")
        } else {
            (make_model[0], make_model[1])
        };

        MetaHeader {
            make: make.to_string(),
            model: model.to_string(),
            min_price,
            max_price,
            total_number,
            ..Default::default()
        }
    }

    pub fn from_string(raw: &str, meta_type: String, dealer: String) -> Self {
        let meta = extract_ascii_latin(raw);
        let re = regex::Regex::new(r" {2,}").unwrap();
        let split: Vec<&str> = re.split(meta.trim()).collect();
        for s in split.clone() {
            info!("split: {}", s);
        }
        let timestamp = chrono::Utc::now().timestamp().to_string();
        if split.len() <= 4 {
            let min_price = split[0].replace(' ', "").parse::<u32>().unwrap_or(0);
            let max_price = split[1].replace(' ', "").parse::<u32>().unwrap_or(0);
            let total_number = split[2].replace(' ', "").parse::<u32>().unwrap_or(0);
            return MetaHeader {
                timestamp,
                meta_type,
                make: "ALL".to_string(),
                model: "ALL".to_string(),
                min_price,
                max_price,
                total_number,
                created_on: chrono::Local::now().format("%Y-%m-%d").to_string(),
                dealer,
            };
        }

        let make_model: Vec<&str> = split[0].split_whitespace().collect();

        let (make, model) = if make_model.len() == 1 {
            (make_model[0], "")
        } else {
            (make_model[0], make_model[1])
        };

        let min = split[1].replace(' ', "").parse::<u32>().unwrap_or(0);
        let max = split[2].replace(' ', "").parse::<u32>().unwrap_or(0);
        let total_number = split[3].replace(' ', "").parse::<u32>().unwrap_or(0);

        MetaHeader {
            timestamp,
            meta_type,
            make: make.to_string(),
            model: model.to_string(),
            min_price: min,
            max_price: max,
            total_number,
            created_on: chrono::Local::now().format("%Y-%m-%d").to_string(),
            dealer,
        }
    }

    pub fn page_numbers(&self) -> u32 {
        let mut pages = self.total_number / 20;
        if self.total_number % 20 > 0 {
            pages += 1;
        }
        pages
    }
}
