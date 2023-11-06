use data_scraper::services::mobile_bg_scraper::{scrape, update};
use data_scraper::utils::helpers::configure_log4rs;
use data_scraper::LOG_CONFIG;
use log::{error, info};

#[tokio::main]
async fn main() {
    configure_log4rs(&LOG_CONFIG);
    info!("Starting scraper");
    let args = std::env::args().collect::<Vec<String>>();
    info!("Using arguments: {:?}", args);
    if args.len() == 2 && args[1] == "update" {
        info!("Updating scraped data...");
        update().await.unwrap_or_else(|e| {
            error!("Failed to update data: {}", e);
        });
    } else {
        info!("Scraping the latest adverts...");
        scrape().await.unwrap_or_else(|e| {
            error!("Failed to scrape data: {}", e);
        });
    }
    info!("Scraper finished");
}
