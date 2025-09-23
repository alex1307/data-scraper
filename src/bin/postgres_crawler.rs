use std::fmt::Debug;
use std::sync::Arc;

use data_scraper::constants::URL::{
    AUTOUNCLE_CH_URL, AUTOUNCLE_DE_URL, AUTOUNCLE_FR_URL, AUTOUNCLE_IT_URL, AUTOUNCLE_NL_URL,
    AUTOUNCLE_PL_URL, AUTOUNCLE_RO_URL, MOBILE_BG_URL,
};
#[cfg(feature = "kafka")]
use data_scraper::kafka::KafkaConsumer::{consumeMobileDeJsons, processMessages};
#[cfg(feature = "kafka")]
use data_scraper::kafka::MOBILE_DE_TOPIC;
#[cfg(feature = "kafka")]
use data_scraper::kafka::broker;

use data_scraper::LOG_CONFIG;
use data_scraper::model::Search::Search;
use data_scraper::model::VehicleDataModel::DownloadStatus;
use data_scraper::scraper::AutouncleScraper;
use data_scraper::scraper::BrowserController::BrowserController;

use data_scraper::scraper::VehicleTraits::VehicleScrapeTrait;
use data_scraper::services::ScraperAppVehicleService;
use data_scraper::services::SearchBuilder::{
    CRAWLER_AUTOUNCLE_CH, CRAWLER_AUTOUNCLE_DE, CRAWLER_AUTOUNCLE_IT, CRAWLER_AUTOUNCLE_PL,
    ID_AUTOUNCLE_CH_START, ID_AUTOUNCLE_DE_START, ID_AUTOUNCLE_FR, ID_AUTOUNCLE_IT_START,
    ID_AUTOUNCLE_NL_START, ID_AUTOUNCLE_PL_START, ID_AUTOUNCLE_RO_START, ID_MOBILE_BG_START,
    build_autouncle_searches,
};
use data_scraper::utils::files::{DATA_DIR, create_all_files};
use data_scraper::writer::sink::SinkType; // Ensure SinkType includes the Protobuf variant or adjust accordingly
use data_scraper::{
    scraper::MobileBgScraper::MobileBGScraper,
    services::SearchBuilder::{
        CRAWLER_AUTOUNCLE_FR, CRAWLER_AUTOUNCLE_NL, CRAWLER_AUTOUNCLE_RO, CRAWLER_MOBILE_BG,
        build_mobile_bg_all_searches,
    },
    utils::helpers::configure_log4rs,
};

use log::{error, info};

use clap::{Args, Parser, Subcommand, command};

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
    #[arg(short, long, default_value = "data")]
    dir: Option<String>,
    #[arg(short = 'o', long, default_value = "csv")]
    sink_type: String,
    #[arg(short = 't', long)]
    topic: Option<String>,
    #[arg(short = 'c', default_value = "false")]
    #[clap(value_parser)]
    use_chrome: Option<bool>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Scrape(CrawlerArgs),
    Puppeteer(CrawlerArgs),
}
#[tokio::main(flavor = "current_thread")]
async fn main() {
    configure_log4rs(&LOG_CONFIG);
    dotenvy::dotenv().ok();
    info!("Starting data scraper ==1==...");
    let command = Cli::parse();
    if let Commands::Scrape(args) = &command.command {
        DATA_DIR
            .set(args.dir.clone().unwrap_or("data".to_string()))
            .unwrap_or_else(|_| panic!("Failed to set DATA_DIR"));
    }
    match command.command {
        Commands::Scrape(args) => {
            let source = args.source.clone();
            let use_chrome = args.use_chrome.unwrap_or(false);
            info!("sink type: {}", args.sink_type);
            let sink_type = match args.sink_type.as_str() {
                "csv" => SinkType::CsvFile,
                "protobuf" => SinkType::ProtobufFile,
                "kafka" => SinkType::Kafka,
                "postgres" => SinkType::PostgresDB,
                _ => {
                    error!("Invalid sink type: {}", args.sink_type);
                    return; // Or another suitable error handling mechanism
                }
            };
            if SinkType::Kafka != sink_type {
                create_all_files(sink_type.clone());
            }
            //create files if not exist BASE_INFO_CSV_FILE_NAME

            run_vehicle_crawler(source, 1, sink_type, use_chrome).await;
        }
        Commands::Puppeteer(args) => {
            let sink_type = match args.sink_type.as_str() {
                "csv" => SinkType::CsvFile,
                "protobuf" => SinkType::ProtobufFile,
                "kafka" => SinkType::Kafka,
                "postgres" => SinkType::PostgresDB,
                _ => {
                    error!("Invalid sink type: {}", args.sink_type);
                    return; // Or another suitable error handling mechanism
                }
            };
            if SinkType::PostgresDB != sink_type && SinkType::Kafka != sink_type {
                //if data dir does not exist, create it
                create_all_files(sink_type.clone());
            }
            info!("Puppeteer command is not implemented yet");
            #[cfg(feature = "kafka")]
            run_consumers(broker(), sink_type).await;
            #[cfg(not(feature = "kafka"))]
            {
                error!("Kafka feature is not enabled. Cannot run consumer.");
            }
        }
    }
}

async fn run_vehicle_crawler(
    crawler: String,
    threads: usize,
    sink_type: SinkType,
    use_chrome: bool,
) {
    let browser = if use_chrome {
        Some(Arc::new(BrowserController::new().await.unwrap()))
    } else {
        None
    };
    let filter = vec![];
    if crawler == CRAWLER_MOBILE_BG {
        let searches = filter_searches(&crawler, filter);

        let crawler = MobileBGScraper::new(MOBILE_BG_URL, 250);
        let searches = searches.chunks(threads);
        info!("Starting mobile.bg with #{} searches", searches.len());
        vehicle_log_and_search(searches, crawler, sink_type, browser).await;
    } else {
        let url = match crawler.as_str() {
            CRAWLER_AUTOUNCLE_FR => AUTOUNCLE_FR_URL,
            CRAWLER_AUTOUNCLE_NL => AUTOUNCLE_NL_URL,
            CRAWLER_AUTOUNCLE_RO => AUTOUNCLE_RO_URL,
            CRAWLER_AUTOUNCLE_CH => AUTOUNCLE_CH_URL,
            CRAWLER_AUTOUNCLE_PL => AUTOUNCLE_PL_URL,
            CRAWLER_AUTOUNCLE_DE => AUTOUNCLE_DE_URL,
            CRAWLER_AUTOUNCLE_IT => AUTOUNCLE_IT_URL,
            _ => {
                error!("Invalid crawler: {}", crawler);
                return;
            }
        };
        let searches = filter_searches(&crawler, filter);
        let crawler = AutouncleScraper::AutouncleScraper::new(url, "page", &crawler, 250);
        let searches = searches.chunks(threads);
        vehicle_log_and_search(searches, crawler, sink_type, browser).await;
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

async fn vehicle_log_and_search<S>(
    searches: std::slice::Chunks<'_, Search>,
    crawler: S,
    sink_type: SinkType,
    browser: Option<Arc<BrowserController>>,
) where
    S: VehicleScrapeTrait + Clone + Send + 'static,
{
    let mut listed = 0;
    let mut actual = 0;
    let mut chunk_counter = 0;
    let mut counter = 0;

    for search in searches {
        chunk_counter += 1;
        let vsearch = search.to_vec();
        let searches = vsearch.to_vec();
        if let Ok(statuses) = ScraperAppVehicleService::download_list_data(
            crawler.clone(),
            searches,
            sink_type.clone(),
            browser.clone(),
        )
        .await
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

#[cfg(feature = "kafka")]
async fn run_consumers(broker: String, sink_type: SinkType) {
    let task = tokio::spawn(async move {
        let group = "mobile_de_group_4";
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
