use data_scraper::services::ScraperAppService::lets_scrape;
use data_scraper::utils::helpers::configure_log4rs;
use data_scraper::LOG_CONFIG;

use log::info;

#[tokio::main]
async fn main() {
    configure_log4rs(&LOG_CONFIG);
    let args = std::env::args().collect::<Vec<String>>();
    info!("Starting crawler: {:?}", args);
    if args.len() < 2 {
        println!("Usage: ./crawler cars.bg or mobile.bg");
        return;
    }

    let crawler_name = &args[1];
    let crawler = match crawler_name.to_lowercase().as_str() {
        "mobile.bg" => lets_scrape("mobile.bg").await,
        _ => lets_scrape("cars.bg").await,
    };

    if let Ok(()) = crawler {
        println!("Success");
    } else {
        println!("Scraping failed.");
    }
}
