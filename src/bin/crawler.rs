use std::collections::HashMap;

use std::fmt::Debug;

use data_scraper::kafka::KafkaConsumer::{consumeCarGrHtmlPages, consumeMobileDeJsons};
use data_scraper::kafka::{broker, CARS_GR_TOPIC, MOBILE_DE_TOPIC};

use data_scraper::model::Search::Search;
use data_scraper::model::VehicleDataModel::{BasicT, ChangeLogT, DetailsT, PriceT};
use data_scraper::scraper::AutouncleFRScraper::AutouncleFRScraper;
use data_scraper::scraper::AutouncleNLScraper::AutouncleNLScraper;
use data_scraper::scraper::Traits::{ScrapeListTrait, ScraperTrait};
use data_scraper::services::SearchBuilder::{
    build_autouncle_searches, ID_AUTOUNCLE_FR, ID_AUTOUNCLE_NL_START, ID_AUTOUNCLE_RO_START,
    ID_CARS_BG_START, ID_MOBILE_BG_START,
};
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
    Scrape(CrawlerArgs),
    Puppeteer,
}
#[tokio::main]
async fn main() {
    configure_log4rs(&LOG_CONFIG);
    let command = Cli::parse();

    match command.command {
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
        let searches = build_mobile_bg_all_searches("https://www.mobile.bg/obiavi/avtomobili-dzhipove/{engine}/{gearbox}{page}/ot-{yearFrom}/do-{yearTo}{page}?f24=2&&engine_power={powerFrom}&engine_power1={powerTo}{priceFrom}{priceTo}", ID_MOBILE_BG_START);
        let converted: Vec<Search> = searches.iter().map(|x| Search::from(x.clone())).collect();
        for c in converted {
            info!("Search: {:?}", c);
        }
        let crawler = MobileBGScraper::new("https://www.mobile.bg/pcgi/mobile.cgi?", 250);
        let searches = searches.chunks(threads);
        log_and_search(searches, crawler).await;
    } else if crawler == CRAWLER_AUTOUNCLE_FR {
        let searches = build_autouncle_searches(
            "https://www.autouncle.fr/en/cars_search?",
            "[5]",
            ID_AUTOUNCLE_FR,
        );
        let converted: Vec<Search> = searches.iter().map(|x| Search::from(x.clone())).collect();
        for c in converted {
            info!("Search: {:?}", c);
        }
        info!("Starting autouncle.fr with #{} searches", searches.len());
        let crawler = AutouncleFRScraper::new("https://www.autouncle.fr/en/cars_search?", 250);
        let searches = searches.chunks(threads);
        info!("Starting autouncle.fr with #{} searches", searches.len());
        log_and_search(searches, crawler).await;
    } else if crawler == CRAWLER_AUTOUNCLE_NL {
        let searches = build_autouncle_searches(
            "https://www.autouncle.nl/en/cars_search?",
            "[5]",
            ID_AUTOUNCLE_NL_START,
        );
        let converted: Vec<Search> = searches.iter().map(|x| Search::from(x.clone())).collect();
        for c in converted {
            info!("Search: {:?}", c);
        }
        info!("Starting autouncle.nl with #{} searches", searches.len());
        let crawler = AutouncleNLScraper::new("https://www.autouncle.nl/en/cars_search?", 250);
        let searches = searches.chunks(threads);
        info!("Starting autouncle.nl with #{} searches", searches.len());
        log_and_search(searches, crawler).await;
    } else if crawler == CRAWLER_AUTOUNCLE_RO {
        let searches = build_autouncle_searches(
            "https://www.autouncle.ro/en/cars_search?",
            "[5]",
            ID_AUTOUNCLE_RO_START,
        );
        let converted: Vec<Search> = searches.iter().map(|x| Search::from(x.clone())).collect();
        for c in converted {
            info!("Search: {:?}", c);
        }
        info!("Starting autouncle.ro with #{} searches", searches.len());
        let crawler = AutouncleROScraper::new("https://www.autouncle.ro/en/cars_search?", 250);
        let searches = searches.chunks(threads);
        info!("Starting autouncle.ro with #{} searches", searches.len());
        log_and_search(searches, crawler).await;
    } else if crawler == CRAWLER_CARS_BG {
        let searches =
            build_cars_bg_all_searches("https://www.cars.bg/carslist.php?", ID_CARS_BG_START);
        let converted: Vec<Search> = searches.iter().map(|x| Search::from(x.clone())).collect();
        for c in converted {
            info!("Search: {:?}", c);
        }
        let crawler = CarsBGScraper::new("https://www.cars.bg/carslist.php?", 250);
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
        let searches = vsearch
            .iter()
            .map(|x| Search::from(x.clone()))
            .collect::<Vec<Search>>();
        if let Ok(statuses) = download_list_data(crawler.clone(), searches).await {
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
