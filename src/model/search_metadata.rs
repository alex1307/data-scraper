use futures::executor::block_on;
use log::info;
use serde::{Deserialize, Serialize};

use crate::{
    scraper::{
        agent::{get_header_data, get_pages, get_pages_async, slink},
        utils::extract_ascii_latin,
    },
    utils::mobile_search_url,
    LISTING_URL, TIMESTAMP,
};

use super::{
    enums::{Dealer, SaleType},
    traits::{Header, Identity},
};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SearchMetadata {
    pub slink: String,
    pub timestamp: i64,
    pub total_number: u32,
    pub min_price: u32,
    pub max_price: u32,
    pub dealer: Dealer,
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
            "dealer",
            "sale_type",
            "total_number",
            "min_price",
            "max_price",
        ]
    }
}

pub fn statistic() -> Vec<SearchMetadata> {
    let dealers_all = search(Dealer::DEALER, SaleType::NONE);
    let private_all = search(Dealer::PRIVATE, SaleType::NONE);
    let all = search(Dealer::ALL, SaleType::NONE);
    vec![all, dealers_all, private_all]
}

pub fn searches() -> Vec<SearchMetadata> {
    let dealer_sold = search(Dealer::DEALER, SaleType::SOLD);
    let dealer_insale = search(Dealer::DEALER, SaleType::INSALE);
    let private_sold = search(Dealer::PRIVATE, SaleType::SOLD);
    let private_insale = search(Dealer::PRIVATE, SaleType::INSALE);
    vec![dealer_sold, private_sold, dealer_insale, private_insale]
}

pub fn search(dealer_type: Dealer, sold: SaleType) -> SearchMetadata {
    block_on({
        asearch(dealer_type, sold)
    })
}

pub async fn asearch(dealer_type: Dealer, sold: SaleType) -> SearchMetadata {
    info!("Searching for: {:?} {:?}", dealer_type, sold);
    let url = mobile_search_url(LISTING_URL, "1", "", dealer_type, sold);
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
        dealer: dealer_type,
        sale_type: sold,
    }
}

impl SearchMetadata {
    pub fn search(dealer_type: Dealer, sold: SaleType) -> Self {
        info!("Searching for: {:?} {:?}", dealer_type, sold);
        let url = mobile_search_url(LISTING_URL, "1", "", dealer_type, sold);
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
            dealer: dealer_type,
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
        model::{enums::Dealer, search_metadata::{search, asearch}},
        utils::configure_log4rs,
    };

   
    #[tokio::test]
    async fn test_search() {
        configure_log4rs("config/loggers/dev_log4rs.yml");
        info!("Test index meta");
        let meta = asearch(Dealer::ALL, crate::model::enums::SaleType::NONE).await;
        let dealer = asearch(Dealer::DEALER, crate::model::enums::SaleType::NONE).await;
        let private = asearch(Dealer::PRIVATE, crate::model::enums::SaleType::NONE).await;
        info!("meta: {:#?}", meta);
        info!("dealer: {:#?}", dealer);
        info!("private: {:#?}", private);
    }
}
