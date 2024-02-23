use std::collections::HashMap;

use data_scraper::kafka::KafkaConsumer::{consumeCarGrHtmlPages, consumeMobileDeJsons};
use data_scraper::kafka::{broker, CARS_GR_TOPIC, MOBILE_DE_TOPIC};
use data_scraper::services::ScraperAppService::{
    download_all, download_autouncle_data, AUTOUNCLE_FR_CRAWLER, AUTOUNCLE_NL_CRAWLER,
    AUTOUNCLE_RO_CRAWLER, MOBILE_BG_CRAWLER,
};
use data_scraper::services::SearchBuilder::{
    build_autouncle_fr_searches, build_autouncle_nl_searches, build_autouncle_ro_searches,
    build_mobile_bg_all_searches,
};

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
    dir: Option<String>,
    topic: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    ScrapeAll(CrawlerArgs),
    ScrapeNew(CrawlerArgs),
    InitSearch(CrawlerArgs),
    ReadDir(CrawlerArgs),
}
#[tokio::main]
async fn main() {
    configure_log4rs(&LOG_CONFIG);
    //rull_all().await;
    run().await;
}

async fn rull_all() {
    let broker = broker();
    let conuser_task = tokio::spawn(run_consumers(broker.clone()));
    let scraper_task = tokio::spawn(run_scrapers());
    let (r1, r2) = tokio::join!(conuser_task, scraper_task);
    if r1.is_ok() {
        info!("Consumer task finished");
    } else {
        info!("Consumer task failed");
    }

    if r2.is_ok() {
        info!("Scraper task finished");
    } else {
        info!("Scraper task failed");
    }

    info!("All tasks finished");
}

async fn run() {
    //let searches: Vec<HashMap<String, String>> = build_mobile_bg_all_searches();
    //let task = tokio::spawn(download_all("mobile.bg"));
    let task = tokio::spawn(download_all("mobile.bg"));

    let (result,) = tokio::join!(task);
    if result.is_ok() {
        info!("task just finished successfully");
    } else {
        info!("task failed");
    }
    info!("The scraper finished. Waiting for 24 hours....");
    tokio::time::sleep(tokio::time::Duration::from_secs(60 * 60 * 24)).await;
}

async fn run_scrapers() {
    let ro_searches: Vec<HashMap<String, String>> = build_autouncle_ro_searches();
    let nl_searches: Vec<HashMap<String, String>> = build_autouncle_nl_searches();
    let fr_searches: Vec<HashMap<String, String>> = build_autouncle_fr_searches();
    loop {
        info!("Running all scrapers");
        let task1 = tokio::spawn(download_all("cars.bg"));
        let task2 = tokio::spawn(download_all("mobile.bg"));
        let task3 = tokio::spawn(download_autouncle_data(
            AUTOUNCLE_RO_CRAWLER.clone(),
            ro_searches.clone(),
        ));
        let task4 = tokio::spawn(download_autouncle_data(
            AUTOUNCLE_NL_CRAWLER.clone(),
            nl_searches.clone(),
        ));

        let task5 = tokio::spawn(download_autouncle_data(
            AUTOUNCLE_FR_CRAWLER.clone(),
            fr_searches.clone(),
        ));

        let (r1, r2, r3, r4, r5) = tokio::join!(task1, task2, task3, task4, task5);
        if r1.is_ok() {
            info!("cars.bg finished");
        } else {
            info!("cars.bg failed");
        }
        if r2.is_ok() {
            info!("mobile.bg finished");
        } else {
            info!("mobile.bg failed");
        }
        if r3.is_ok() {
            info!("autouncle.ro finished");
        } else {
            info!("autouncle.ro failed");
        }
        if r4.is_ok() {
            info!("autouncle.nl finished");
        } else {
            info!("autouncle.nl failed");
        }
        if r5.is_ok() {
            info!("autouncle.fr finished");
        } else {
            info!("autouncle.fr failed");
        }
        info!("All scrapers finished. Waiting for 24 hours....");
        tokio::time::sleep(tokio::time::Duration::from_secs(60 * 60 * 24)).await;
    }
}

async fn run_consumers(broker: String) {
    let broker_gr = broker.clone();
    let task1 = tokio::spawn(async move {
        consumeCarGrHtmlPages(&broker_gr, "car.gr.group", CARS_GR_TOPIC).await
    });
    let task2 = tokio::spawn(async move {
        consumeMobileDeJsons(&broker, "mobile.de.group", MOBILE_DE_TOPIC).await
    });
    let (r1, r2) = tokio::join!(task1, task2);
    if r1.is_ok() {
        info!("car.gr consumer finished");
    } else {
        info!("car.gr consumer failed");
    }
    if r2.is_ok() {
        info!("mobile.de consumer finished");
    } else {
        info!("mobile.de consumer failed");
    }
}
