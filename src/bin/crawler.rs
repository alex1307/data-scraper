use data_scraper::services::ScraperAppService::{download_all, download_new_vehicles};
use data_scraper::utils::helpers::configure_log4rs;
use data_scraper::LOG_CONFIG;

use log::info;

use clap::{command, Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Args, Debug)]
struct CrawlerArgs {
    source: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    ScrapeAll(CrawlerArgs),
    ScrapeNew(CrawlerArgs),
}

#[tokio::main]
async fn main() {
    configure_log4rs(&LOG_CONFIG);
    let command = Cli::parse();
    let cmd = &command.command;

    let crawler = match cmd {
        Commands::ScrapeAll(args) => {
            info!("Starting crawler: {:?}", command);
            info!("cmd: {:?}", cmd);
            info!("args: {:?}", args);
            if args.source.is_empty() {
                info!("Usage: ./crawler cars.bg or mobile.bg");
                return;
            }
            download_all(&args.source).await
        }
        Commands::ScrapeNew(args) => {
            info!("Starting crawler: {:?}", command);
            info!("cmd: {:?}", cmd);
            info!("args: {:?}", args);
            if args.source.is_empty() {
                info!("Usage: ./crawler cars.bg or mobile.bg");
                return;
            }
            download_new_vehicles(&args.source).await
        }
    };

    if let Ok(()) = crawler {
        info!("Success");
    } else {
        info!("Scraping failed.");
    }
}
