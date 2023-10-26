use data_scraper::cmd::scrape_listing;

#[tokio::main]
async fn main() {
    scrape_listing().await;
}
