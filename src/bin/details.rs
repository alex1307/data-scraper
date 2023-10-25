use data_scraper::{cmd::scrape_details, SEARCH_ALL};

#[tokio::main]
async fn main() {
    scrape_details(SEARCH_ALL.slink.as_str()).await;
}
