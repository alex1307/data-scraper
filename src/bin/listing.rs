use data_scraper::cmd::scrape;

#[tokio::main]
async fn main() {
    scrape().await;
}
