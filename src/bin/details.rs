use data_scraper::{cmd::scrape_details, model::{search_metadata::asearch, enums::{Dealer, SaleType}}};

#[tokio::main]
async fn main() {
    let all = asearch(Dealer::ALL, SaleType::NONE).await;
    scrape_details(all.slink.as_str()).await;
}
