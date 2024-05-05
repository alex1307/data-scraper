use std::fmt::Debug;

use data_scraper::constants::URL::{
    AUTOUNCLE_FR_URL, AUTOUNCLE_NL_URL, AUTOUNCLE_RO_URL, CARS_BG_URL, MOBILE_BG_URL,
};
use data_scraper::kafka::KafkaConsumer::{consumeMobileDeJsons, processMessages};
use data_scraper::kafka::{broker, MOBILE_DE_TOPIC};

use data_scraper::model::Search::Search;
use data_scraper::model::VehicleDataModel::{BasicT, ChangeLogT, DetailsT, DownloadStatus, PriceT};
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
    let statuses = processMessages(&broker(), &crawler, "status_info", 15).await;
    info!("Statuses: {:?}", statuses.len());
    let mut map = std::collections::HashMap::new();
    for status in statuses {
        map.entry(status.source.clone())
            .or_insert_with(Vec::new)
            .push(status);
    }

    if crawler == CRAWLER_MOBILE_BG {
        let filter = if let Some(found) = map.get(&crawler) {
            found.to_vec()
        } else {
            vec![]
        };
        let searches = filter_searches(&crawler, filter);

        let crawler = MobileBGScraper::new(MOBILE_BG_URL, 250);
        let searches = searches.chunks(threads);
        log_and_search(searches, crawler).await;
    } else if crawler == CRAWLER_AUTOUNCLE_FR {
        let filter = if let Some(found) = map.get(&crawler) {
            found.to_vec()
        } else {
            vec![]
        };
        let searches = filter_searches(&crawler, filter);
        let crawler = AutouncleFRScraper::new(AUTOUNCLE_FR_URL, 250);
        let searches = searches.chunks(threads);
        info!("Starting autouncle.fr with #{} searches", searches.len());
        log_and_search(searches, crawler).await;
    } else if crawler == CRAWLER_AUTOUNCLE_NL {
        let filter = if let Some(found) = map.get(&crawler) {
            found.to_vec()
        } else {
            vec![]
        };
        let searches = filter_searches(&crawler, filter);
        let crawler = AutouncleNLScraper::new(AUTOUNCLE_NL_URL, 250);
        info!("Starting autouncle.nl with #{} searches", searches.len());
        let searches = searches.chunks(threads);
        log_and_search(searches, crawler).await;
    } else if crawler == CRAWLER_AUTOUNCLE_RO {
        let filter = if let Some(found) = map.get(&crawler) {
            found.to_vec()
        } else {
            vec![]
        };
        let searches = filter_searches(&crawler, filter);
        let crawler = AutouncleROScraper::new(AUTOUNCLE_RO_URL, 250);
        let searches = searches.chunks(threads);
        info!("Starting autouncle.ro with #{} searches", searches.len());
        log_and_search(searches, crawler).await;
    } else if crawler == CRAWLER_CARS_BG {
        let filter = if let Some(found) = map.get(&crawler) {
            found.to_vec()
        } else {
            vec![]
        };
        let searches = filter_searches(&crawler, filter);
        let crawler = CarsBGScraper::new(CARS_BG_URL, 250);
        let searches = searches.chunks(threads);
        log_and_search(searches, crawler).await;
    } else {
        error!("Invalid crawler: {}", crawler);
    }
}
fn filter_searches(source: &str, filter: Vec<DownloadStatus>) -> Vec<Search> {
    let searches = match source {
        CRAWLER_AUTOUNCLE_FR => build_autouncle_searches(AUTOUNCLE_FR_URL, "[5]", ID_AUTOUNCLE_FR),
        CRAWLER_AUTOUNCLE_NL => {
            build_autouncle_searches(AUTOUNCLE_NL_URL, "[5]", ID_AUTOUNCLE_NL_START)
        }
        CRAWLER_AUTOUNCLE_RO => {
            build_autouncle_searches(AUTOUNCLE_RO_URL, "[5]", ID_AUTOUNCLE_RO_START)
        }
        CRAWLER_CARS_BG => build_cars_bg_all_searches(CARS_BG_URL, ID_CARS_BG_START),
        CRAWLER_MOBILE_BG => build_mobile_bg_all_searches(MOBILE_BG_URL, ID_MOBILE_BG_START),

        _ => vec![],
    };

    let mut converted: Vec<Search> = searches.iter().map(|x| Search::from(x.clone())).collect();
    for f in filter {
        let search = converted
            .iter()
            .find(|x| x.url == f.url || x.hash == f.hash);
        if let Some(s) = search {
            let index = converted.iter().position(|x| x.id == s.id).unwrap();
            converted.remove(index);
        }
    }
    info!(
        "Starting autouncle.ro with #{} searches and filtered: {}",
        searches.len(),
        converted.len()
    );
    converted
}

async fn log_and_search<S, T>(searches: std::slice::Chunks<'_, Search>, crawler: S)
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
        let searches = vsearch.to_vec();
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
    let task = tokio::spawn(async move {
        consumeMobileDeJsons(&broker, "mobile.de.group", MOBILE_DE_TOPIC).await
    });
    let r1 = tokio::spawn(task).await;
    if r1.is_ok() {
        info!("car.gr consumer finished");
    } else {
        info!("car.gr consumer failed");
    }
}
