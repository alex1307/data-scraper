use data_scraper::services::mobile_bg_scraper::scrape;
use data_scraper::utils::helpers::configure_log4rs;
use data_scraper::LOG_CONFIG;
use log::{error, info};

#[tokio::main]
async fn main() {
    configure_log4rs(&LOG_CONFIG);
    info!("Starting scraper");

    scrape().await.unwrap_or_else(|e| {
        error!("Failed to scrape data: {}", e);
    });
}
