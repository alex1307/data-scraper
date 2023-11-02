use log::info;
use serde::{Deserialize, Serialize};

use crate::{
    scraper::mobile_bg::{get_header_data, get_pages, get_pages_async, slink},
    utils::helpers::{extract_ascii_latin, mobile_search_url},
    LISTING_URL, TIMESTAMP,
};

use super::{
    enums::SaleType,
    traits::{Header, Identity},
};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SearchMetadata {
    pub slink: String,
    pub timestamp: i64,
    pub total_number: u32,
    pub min_price: u32,
    pub max_price: u32,
    pub sale_type: SaleType,
}

impl Identity for SearchMetadata {
    fn get_id(&self) -> String {
        self.timestamp.to_string()
    }
}

impl Header for SearchMetadata {
    fn header() -> Vec<&'static str> {
        vec![
            "slink",
            "timestamp",
            "sale_type",
            "min_price",
            "max_price",
            "total_number",
        ]
    }
}

pub async fn astatistic() -> Vec<SearchMetadata> {
    let insale = asearch(SaleType::INSALE, 0, 0).await;
    let sold = asearch(SaleType::SOLD, 0, 0).await;
    let all = asearch(SaleType::NONE, 0, 0).await;
    vec![all, insale, sold]
}

pub async fn asearches() -> Vec<SearchMetadata> {
    let sold = asearch(SaleType::SOLD, 0, 0).await;
    let insale_5000 = asearch(SaleType::INSALE, 1_001, 5000).await;
    let insale_10_000 = asearch(SaleType::INSALE, 5001, 10_000).await;
    let insale_15_000 = asearch(SaleType::INSALE, 10_001, 15_000).await;
    let insale_20_000 = asearch(SaleType::INSALE, 15_001, 20_000).await;
    let insale_30_000 = asearch(SaleType::INSALE, 20_001, 30_000).await;
    let insale_30_000_up = asearch(SaleType::INSALE, 30_001, 0).await;
    vec![
        sold,
        insale_5000,
        insale_10_000,
        insale_15_000,
        insale_20_000,
        insale_30_000,
        insale_30_000_up,
    ]
}

pub async fn asearch(sold: SaleType, min: i32, max: i32) -> SearchMetadata {
    info!(
        "Searching for sale type: {:?}, min price {} max price{}",
        sold, min, max
    );
    let url = mobile_search_url(LISTING_URL, "1", "", sold, min, max);
    info!("url: {}", url);
    let html = get_pages_async(&url).await.unwrap();
    // info!("content: {}", html);
    let slink = slink(&html);
    let content = get_header_data(&html).unwrap();
    let meta = extract_ascii_latin(&content);
    let re = regex::Regex::new(r" {2,}").unwrap();
    let split: Vec<&str> = re.split(meta.trim()).collect();
    info!("split: {:?}", split);
    let min_price = split[0].replace(' ', "").parse::<u32>().unwrap_or(0);
    let max_price = split[1].replace(' ', "").parse::<u32>().unwrap_or(0);
    let total_number = split[2].replace(' ', "").parse::<u32>().unwrap_or(0);
    SearchMetadata {
        slink,
        min_price,
        max_price,
        total_number,
        timestamp: *TIMESTAMP,
        sale_type: sold,
    }
}

impl SearchMetadata {
    pub fn search(sold: SaleType, min_price: i32, max_price: i32) -> Self {
        info!(
            "Searching for sale type: {:?}, min price {}, max price {}",
            sold, min_price, max_price
        );
        let url = mobile_search_url(LISTING_URL, "1", "", sold, min_price, max_price);
        info!("url: {}", url);
        let html = get_pages(&url).unwrap();
        // info!("content: {}", html);
        let slink = slink(&html);
        let content = get_header_data(&html).unwrap();
        let meta = extract_ascii_latin(&content);
        let re = regex::Regex::new(r" {2,}").unwrap();
        let split: Vec<&str> = re.split(meta.trim()).collect();
        info!("split: {:?}", split);
        let min_price = split[0].replace(' ', "").parse::<u32>().unwrap_or(0);
        let max_price = split[1].replace(' ', "").parse::<u32>().unwrap_or(0);
        let total_number = split[2].replace(' ', "").parse::<u32>().unwrap_or(0);
        SearchMetadata {
            slink,
            min_price,
            max_price,
            total_number,
            timestamp: *TIMESTAMP,
            sale_type: sold,
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

#[cfg(test)]
mod test {
    use log::info;

    use crate::{
        model::search_metadata::{asearches, astatistic},
        utils::helpers::configure_log4rs,
    };

    #[tokio::test]
    async fn test_search() {
        configure_log4rs("config/loggers/dev_log4rs.yml");
        info!("Test index meta");
        let stats = astatistic().await;
        let searches = asearches().await;
        assert_eq!(3, stats.len());
        assert_eq!(4, searches.len());
    }
}
