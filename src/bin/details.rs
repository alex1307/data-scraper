use data_scraper::cmd::scrape_details;

#[tokio::main]
async fn main() {
    scrape_details().await;
}
