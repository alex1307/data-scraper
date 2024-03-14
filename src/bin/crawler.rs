use std::collections::HashMap;

use std::fmt::Debug;

use std::vec;

use data_scraper::kafka::KafkaConsumer::{consumeCarGrHtmlPages, consumeMobileDeJsons};
use data_scraper::kafka::{broker, CARS_GR_TOPIC, MOBILE_DE_TOPIC};

use data_scraper::model::VehicleDataModel::{BasicT, ChangeLogT, DetailsT, PriceT};
use data_scraper::scraper::AutouncleFRScraper::AutouncleFRScraper;
use data_scraper::scraper::AutouncleNLScraper::AutouncleNLScraper;
use data_scraper::scraper::Traits::{ScrapeListTrait, ScraperTrait};
use data_scraper::services::SearchBuilder::build_autouncle_searches;
use data_scraper::LOG_CONFIG;
use data_scraper::{
    scraper::{
        AutouncleROScraper::AutouncleROScraper, CarsBgScraper::CarsBGScraper,
        MobileBgScraper::MobileBGScraper,
    },
    services::{
        ScraperAppService::download_list_data,
        SearchBuilder::{
            build_cars_bg_all_searches, build_mobile_bg_all_searches, CRAWLER_AUTOUNCLE_FR,
            CRAWLER_AUTOUNCLE_NL, CRAWLER_AUTOUNCLE_RO, CRAWLER_CARS_BG, CRAWLER_MOBILE_BG,
        },
    },
    utils::helpers::configure_log4rs,
};

use log::{error, info};

use clap::{command, Args, Parser, Subcommand};
use rand::seq::SliceRandom;
use serde::Serialize;

pub const CHUNK_SIZE: usize = 4;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Args, Debug)]
struct CrawlerArgs {
    source: String,
    threads: Option<usize>,
    dir: Option<String>,
    topic: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    ScrapeAll,
    Scrape(CrawlerArgs),
    Puppeteer,
}
#[tokio::main]
async fn main() {
    configure_log4rs(&LOG_CONFIG);
    let command = Cli::parse();

    match command.command {
        Commands::ScrapeAll => run().await,
        Commands::Scrape(args) => {
            let source = args.source.clone();
            let threads = args.threads.unwrap_or(1);
            run_crawler(source, threads).await;
        }
        Commands::Puppeteer => {
            info!("Puppeteer command is not implemented yet");
            run_consumers(broker()).await;
        }
    }
}

async fn run_crawler(crawler: String, threads: usize) {
    if crawler == CRAWLER_MOBILE_BG {
        let searches = build_mobile_bg_all_searches();
        let crawler = MobileBGScraper::new("https://www.mobile.bg/pcgi/mobile.cgi?", 250);
        let searches = searches.chunks(threads);
        log_and_search(searches, crawler).await;
    } else if crawler == CRAWLER_AUTOUNCLE_FR {
        let searches = build_autouncle_searches("[5]");
        info!("Starting autouncle.fr with #{} searches", searches.len());
        let crawler = AutouncleFRScraper::new("https://www.autouncle.fr/en/cars_search?", 250);
        let searches = searches.chunks(threads);
        info!("Starting autouncle.fr with #{} searches", searches.len());
        log_and_search(searches, crawler).await;
    } else if crawler == CRAWLER_AUTOUNCLE_NL {
        let searches = build_autouncle_searches("[5]");
        info!("Starting autouncle.nl with #{} searches", searches.len());
        let crawler = AutouncleNLScraper::new("https://www.autouncle.nl/en/cars_search?", 250);
        let searches = searches.chunks(threads);
        info!("Starting autouncle.nl with #{} searches", searches.len());
        log_and_search(searches, crawler).await;
    } else if crawler == CRAWLER_AUTOUNCLE_RO {
        let searches = build_autouncle_searches("[5]");
        info!("Starting autouncle.ro with #{} searches", searches.len());
        let crawler = AutouncleROScraper::new("https://www.autouncle.ro/en/cars_search?", 250);
        let searches = searches.chunks(threads);
        info!("Starting autouncle.ro with #{} searches", searches.len());
        log_and_search(searches, crawler).await;
    } else if crawler == CRAWLER_CARS_BG {
        let searches = build_cars_bg_all_searches();
        let crawler = CarsBGScraper::new("https://www.cars.bg", 250);
        let searches = searches.chunks(threads);
        log_and_search(searches, crawler).await;
    } else {
        error!("Invalid crawler: {}", crawler);
    }
}

async fn log_and_search<S, T>(searches: std::slice::Chunks<'_, HashMap<String, String>>, crawler: S)
where
    S: ScraperTrait + ScrapeListTrait<T> + Clone + Send + 'static,
    T: BasicT + DetailsT + PriceT + ChangeLogT + Send + Serialize + Clone + Debug + 'static,
{
    let mut listed = 0;
    let mut actual = 0;
    let mut chunk_counter = 0;
    let mut counter = 0;
    for search in searches {
        chunk_counter += 1;
        let vsearch = search.to_vec();
        if let Ok(statuses) = download_list_data(crawler.clone(), vsearch).await {
            for s in statuses {
                listed += s.listed;
                actual += s.actual;
                counter += 1;
            }
            info!(
                "Listed: {}, Actual: {}, Chunk#: {}, Searches#: {}",
                listed, actual, chunk_counter, counter
            );
        }
    }
}

async fn run() {
    let mut all = vec![];

    let random = to_execution_list(build_mobile_bg_all_searches(), CRAWLER_MOBILE_BG, 10);
    let mobile_bg_all = random.len();
    all.extend(random.clone());

    let random = to_execution_list(build_autouncle_searches("[5]"), CRAWLER_AUTOUNCLE_FR, 4);
    let fr_all = random.len();
    all.extend(random.clone());

    let random = to_execution_list(build_autouncle_searches("[5]"), CRAWLER_AUTOUNCLE_NL, 4);
    let nl_all = random.len();
    all.extend(random.clone());

    let random = to_execution_list(build_autouncle_searches("[5]"), CRAWLER_AUTOUNCLE_RO, 4);
    let ro_all = random.len();
    all.extend(random.clone());

    let random = to_execution_list(build_cars_bg_all_searches(), CRAWLER_CARS_BG, 10);
    let cars_bg_all = random.len();
    all.extend(random.clone());

    all.shuffle(&mut rand::thread_rng());

    let mobile_bg_crawler = MobileBGScraper::new("https://www.mobile.bg/pcgi/mobile.cgi?", 250);
    let ro_crawler = AutouncleROScraper::new("https://www.autouncle.ro/en/cars_search?", 250);
    let nl_crawler = AutouncleNLScraper::new("https://www.autouncle.nl/en/cars_search?", 250);
    let fr_crawler = AutouncleFRScraper::new("https://www.autouncle.fr/en/cars_search?", 250);
    let cars_bg_crawler = CarsBGScraper::new("https://www.cars.bg", 250);

    info!("Starting the scrapers. All serches: {}", all.len());
    let total_number = all.len();
    let mut progress = 0;
    let mut ro_progress = 0;
    let mut nl_progress = 0;
    let mut fr_progress = 0;
    let mut cars_bg_progress = 0;
    let mut mobile_bg_progress = 0;
    for (crawler, searches) in all {
        info!("------->>>>>> Progress <<<<<<<<-------");
        info!("Overall Progress: {}/{}", progress, total_number);
        info!(
            "Progress: mobile.bg: {}/{}",
            mobile_bg_progress, mobile_bg_all
        );
        info!("Progress: autouncle.ro: {}/{}", ro_progress, ro_all);
        info!("Progress: autouncle.nl: {}/{}", nl_progress, nl_all);
        info!("Progress: autouncle.fr: {}/{}", fr_progress, fr_all);
        info!("Progress: cars.bg: {}/{}", cars_bg_progress, cars_bg_all);
        info!("------->>>>>> ******** <<<<<<<<-------");
        if crawler == CRAWLER_MOBILE_BG {
            info!("Starting mobile.bg with #{} searches", searches.len());
            let _ = download_list_data(mobile_bg_crawler.clone(), searches).await;
            progress += 1;
            mobile_bg_progress += 1;

            continue;
        }
        if crawler == CRAWLER_AUTOUNCLE_FR {
            info!("Starting autouncle.fr with #{} searches", searches.len());
            let _ = download_list_data(fr_crawler.clone(), searches).await;
            progress += 1;
            fr_progress += 1;
            continue;
        }
        if crawler == CRAWLER_AUTOUNCLE_NL {
            info!("Starting autouncle.nl with #{} searches", searches.len());
            let _ = download_list_data(nl_crawler.clone(), searches).await;
            progress += 1;
            nl_progress += 1;
            continue;
        }
        if crawler == CRAWLER_AUTOUNCLE_RO {
            info!("Starting autouncle.ro with #{} searches", searches.len());
            let _ = download_list_data(ro_crawler.clone(), searches).await;
            progress += 1;
            ro_progress += 1;
            continue;
        }
        if crawler == CRAWLER_CARS_BG {
            info!("Starting cars.bg with #{} searches", searches.len());
            let _ = download_list_data(cars_bg_crawler.clone(), searches).await;
            progress += 1;
            cars_bg_progress += 1;
            continue;
        }
    }

    info!("The scraper finished. Waiting for 24 hours....");
    tokio::time::sleep(tokio::time::Duration::from_secs(60 * 60 * 24)).await;
}

fn to_execution_list(
    source: Vec<HashMap<String, String>>,
    crawler: &str,
    chunk_size: usize,
) -> Vec<(String, Vec<HashMap<String, String>>)> {
    let chunks = source.chunks(chunk_size);
    let mut random = vec![];
    for chunk in chunks {
        let searches = chunk.to_vec();
        random.push((crawler.to_string(), searches));
    }
    random
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
