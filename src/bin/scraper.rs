use data_scraper::services::node::scrape;
use data_scraper::{utils::configure_log4rs, LOG_CONFIG};
use log::{error, info};

#[tokio::main]
async fn main() {
    configure_log4rs(&LOG_CONFIG);
    info!("Starting scraper");

    scrape().await.unwrap_or_else(|e| {
        error!("Failed to scrape data: {}", e);
    });
}
