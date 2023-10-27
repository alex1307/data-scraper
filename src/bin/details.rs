use data_scraper::{
    cmd::scrape_details,
    model::{
        enums::{Dealer, SaleType},
        search_metadata::asearch,
    },
};

#[tokio::main]
async fn main() {
    let all = asearch(Dealer::ALL, SaleType::NONE).await;
    scrape_details(all.slink.as_str()).await;
}
