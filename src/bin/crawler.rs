use std::fmt::Debug;

use data_scraper::constants::URL::{
    AUTOUNCLE_CH_URL, AUTOUNCLE_DE_URL, AUTOUNCLE_FR_URL, AUTOUNCLE_IT_URL, AUTOUNCLE_NL_URL,
    AUTOUNCLE_PL_URL, AUTOUNCLE_RO_URL, CARS_BG_URL, MOBILE_BG_URL,
};
#[cfg(feature = "kafka")]
use data_scraper::kafka::KafkaConsumer::{consumeMobileDeJsons, processMessages};
use data_scraper::kafka::{MOBILE_DE_TOPIC, broker};

use data_scraper::LOG_CONFIG;
use data_scraper::model::Search::Search;
use data_scraper::model::VehicleDataModel::{BasicT, DetailsT, DownloadStatus, PriceT};
use data_scraper::scraper::AutouncleCHScraper::AutouncleCHScraper;
use data_scraper::scraper::AutouncleFRScraper::AutouncleFRScraper;
use data_scraper::scraper::AutouncleNLScraper::AutouncleNLScraper;
use data_scraper::scraper::AutounclePLScraper::AutounclePLScraper;
use data_scraper::scraper::Traits::{ScrapeListTrait, ScraperTrait};
use data_scraper::services::SearchBuilder::{
    CRAWLER_AUTOUNCLE_CH, CRAWLER_AUTOUNCLE_DE, CRAWLER_AUTOUNCLE_IT, CRAWLER_AUTOUNCLE_PL,
    ID_AUTOUNCLE_CH_START, ID_AUTOUNCLE_DE_START, ID_AUTOUNCLE_FR, ID_AUTOUNCLE_IT_START,
    ID_AUTOUNCLE_NL_START, ID_AUTOUNCLE_PL_START, ID_AUTOUNCLE_RO_START, ID_CARS_BG_START,
    ID_MOBILE_BG_START, build_autouncle_searches,
};
use data_scraper::utils::files::create_all_files;
use data_scraper::writer::sink::SinkType; // Ensure SinkType includes the Protobuf variant or adjust accordingly
use data_scraper::{
    scraper::{AutouncleROScraper::AutouncleROScraper, MobileBgScraper::MobileBGScraper},
    services::{
        ScraperAppService::download_list_data,
        SearchBuilder::{
            CRAWLER_AUTOUNCLE_FR, CRAWLER_AUTOUNCLE_NL, CRAWLER_AUTOUNCLE_RO, CRAWLER_CARS_BG,
            CRAWLER_MOBILE_BG, build_cars_bg_all_searches, build_mobile_bg_all_searches,
        },
    },
    utils::helpers::configure_log4rs,
};

use log::{error, info};

use clap::{Args, Parser, Subcommand, command};

use serde::Serialize;
use uuid::Uuid;

pub const CHUNK_SIZE: usize = 4;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Args, Debug)]
struct CrawlerArgs {
    #[arg(short, long, default_value = "autouncle.ro")]
    source: String,
    #[arg(short, long, default_value = "./data")]
    dir: Option<String>,
    #[arg(short = 'o', long, default_value = "csv")]
    sink_type: String,
    #[arg(short = 't', long)]
    topic: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Scrape(CrawlerArgs),
    Puppeteer(CrawlerArgs),
}
#[tokio::main]
async fn main() {
    configure_log4rs(&LOG_CONFIG);
    let command = Cli::parse();
    match command.command {
        Commands::Scrape(args) => {
            let source = args.source.clone();
            let sink_type = match args.sink_type.as_str() {
                "csv" => SinkType::CsvFile,
                "protobuf" => SinkType::ProtobufFile,
                "kafka" => SinkType::Kafka,
                _ => {
                    error!("Invalid sink type: {}", args.sink_type);
                    return; // Or another suitable error handling mechanism
                }
            };
            if SinkType::Kafka != sink_type {
                //if data dir does not exist, create it
                if let Some(dir) = args.dir {
                    create_all_files(&dir, sink_type.clone());

                    // Check if the file exists
                }
            }
            //create files if not exist BASE_INFO_CSV_FILE_NAME

            run_crawler(source, 1, sink_type).await;
        }
        Commands::Puppeteer(args) => {
            let sink_type = match args.sink_type.as_str() {
                "csv" => SinkType::CsvFile,
                "protobuf" => SinkType::ProtobufFile,
                "kafka" => SinkType::Kafka,
                _ => {
                    error!("Invalid sink type: {}", args.sink_type);
                    return; // Or another suitable error handling mechanism
                }
            };
            if SinkType::Kafka != sink_type {
                //if data dir does not exist, create it
                if let Some(dir) = args.dir {
                    create_all_files(&dir, sink_type.clone());
                }
            }
            info!("Puppeteer command is not implemented yet");
            run_consumers(broker(), sink_type).await;
        }
    }
}

async fn run_crawler(crawler: String, threads: usize, sink_type: SinkType) {
    let filter = vec![];
    if crawler == CRAWLER_MOBILE_BG {
        let searches = filter_searches(&crawler, filter);

        let crawler = MobileBGScraper::new(MOBILE_BG_URL, 250);
        let searches = searches.chunks(threads);
        info!("Starting mobile.bg with #{} searches", searches.len());
        log_and_search(searches, crawler, sink_type).await;
    } else if crawler == CRAWLER_AUTOUNCLE_FR {
        let searches = filter_searches(&crawler, filter);
        let crawler = AutouncleFRScraper::new(AUTOUNCLE_FR_URL, 250);
        let searches = searches.chunks(threads);
        info!("Starting autouncle.fr with #{} searches", searches.len());
        log_and_search(searches, crawler, sink_type).await;
    } else if crawler == CRAWLER_AUTOUNCLE_NL {
        let searches = filter_searches(&crawler, filter);
        let crawler = AutouncleNLScraper::new(AUTOUNCLE_NL_URL, 250);
        info!("Starting autouncle.nl with #{} searches", searches.len());
        let searches = searches.chunks(threads);
        log_and_search(searches, crawler, sink_type).await;
    } else if crawler == CRAWLER_AUTOUNCLE_RO {
        let searches = filter_searches(&crawler, filter);
        let crawler = AutouncleROScraper::new(AUTOUNCLE_RO_URL, 250);
        let searches = searches.chunks(threads);
        info!("Starting autouncle.ro with #{} searches", searches.len());
        log_and_search(searches, crawler, sink_type).await;
    } else if crawler == CRAWLER_AUTOUNCLE_PL {
        let searches = filter_searches(&crawler, filter);
        let crawler = AutounclePLScraper::new(AUTOUNCLE_PL_URL, 250);
        let searches = searches.chunks(threads);
        info!("Starting autouncle.pl with #{} searches", searches.len());
        log_and_search(searches, crawler, sink_type).await;
    } else if crawler == CRAWLER_AUTOUNCLE_CH {
        let searches = filter_searches(&crawler, filter);
        let crawler = AutouncleCHScraper::new(AUTOUNCLE_CH_URL, 250);
        let searches = searches.chunks(threads);
        info!("Starting autouncle.pl with #{} searches", searches.len());
        log_and_search(searches, crawler, sink_type).await;
    } else if crawler == CRAWLER_AUTOUNCLE_DE {
        let searches = filter_searches(&crawler, filter);
        let crawler = AutouncleCHScraper::new(AUTOUNCLE_DE_URL, 250);
        let searches = searches.chunks(threads);
        info!("Starting autouncle.pl with #{} searches", searches.len());
        log_and_search(searches, crawler, sink_type).await;
    } else if crawler == CRAWLER_AUTOUNCLE_IT {
        let searches = filter_searches(&crawler, filter);
        let crawler = AutouncleCHScraper::new(AUTOUNCLE_IT_URL, 250);
        let searches = searches.chunks(threads);
        info!("Starting autouncle.pl with #{} searches", searches.len());
        log_and_search(searches, crawler, sink_type).await;
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
        CRAWLER_AUTOUNCLE_CH => {
            build_autouncle_searches(AUTOUNCLE_CH_URL, "[5]", ID_AUTOUNCLE_CH_START)
        }
        CRAWLER_AUTOUNCLE_PL => {
            build_autouncle_searches(AUTOUNCLE_PL_URL, "[5]", ID_AUTOUNCLE_PL_START)
        }
        CRAWLER_AUTOUNCLE_DE => {
            build_autouncle_searches(AUTOUNCLE_DE_URL, "[5]", ID_AUTOUNCLE_DE_START)
        }
        CRAWLER_AUTOUNCLE_IT => {
            build_autouncle_searches(AUTOUNCLE_IT_URL, "[5]", ID_AUTOUNCLE_IT_START)
        }
        CRAWLER_CARS_BG => build_cars_bg_all_searches(CARS_BG_URL, ID_CARS_BG_START),
        CRAWLER_MOBILE_BG => build_mobile_bg_all_searches(MOBILE_BG_URL, ID_MOBILE_BG_START),

        _ => vec![],
    };
    info!("Converting {} to #{} searches", source, searches.len());
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
        "Starting {} with #{} searches and filtered: {}",
        source,
        searches.len(),
        converted.len()
    );
    converted
}

async fn log_and_search<S, T>(
    searches: std::slice::Chunks<'_, Search>,
    crawler: S,
    sink_type: SinkType,
) where
    S: ScraperTrait + ScrapeListTrait<T> + Clone + Send + 'static,
    T: BasicT + DetailsT + PriceT + Send + Sync + Serialize + Clone + Debug + 'static,
{
    let mut listed = 0;
    let mut actual = 0;
    let mut chunk_counter = 0;
    let mut counter = 0;

    for search in searches {
        chunk_counter += 1;
        let vsearch = search.to_vec();
        let searches = vsearch.to_vec();
        if let Ok(statuses) = download_list_data(crawler.clone(), searches, sink_type.clone()).await
        {
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

async fn run_consumers(broker: String, sink_type: SinkType) {
    let task = tokio::spawn(async move {
        #[cfg(feature = "kafka")]
        {
            consumeMobileDeJsons(&broker, &group, MOBILE_DE_TOPIC, sink_type).await
        }
        #[cfg(not(feature = "kafka"))]
        {
            error!("Kafka feature is not enabled. Cannot run consumer.");
        }
    });
    let r1 = tokio::spawn(task).await;
    if r1.is_ok() {
        info!("car.gr consumer finished");
    } else {
        info!("car.gr consumer failed");
    }
}
