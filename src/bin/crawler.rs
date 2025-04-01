use std::fmt::Debug;
use std::fs::File;
use std::path::Path;

use data_scraper::constants::URL::{
    AUTOUNCLE_CH_URL, AUTOUNCLE_DE_URL, AUTOUNCLE_FR_URL, AUTOUNCLE_IT_URL, AUTOUNCLE_NL_URL,
    AUTOUNCLE_PL_URL, AUTOUNCLE_RO_URL, CARS_BG_URL, MOBILE_BG_URL,
};
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

const VEHICLE_HEADER: &str = "id;source;make;model;title;currency;price;mileage;month;year;engine;gearbox;cc;power_ps;power_kw;search_id;url";
const DETAILS_HEADER: &str = "id;source;location;equipment;seller_name;seller_url;range;consumption_fuel;consumption_kw;co2;days_in_sale";
const PRICES_HEADER: &str =
    "id;source;estimated_price;price;currency;save_difference;overpriced_difference;ranges;rating";

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
                    if let Err(e) = std::fs::create_dir_all(&dir) {
                        error!("Failed to create directory {}: {}", dir, e);
                        // Handle the error appropriately, e.g., return an error, exit with a non-zero code, etc.
                        return; // Or another suitable error handling mechanism
                    }
                    info!("Data directory: {}", dir);
                    // Create the file name with the current date
                    //let file_name = format!("{}/base-info-{}.csv", dir, CREATED_ON);
                    let extension = match sink_type {
                        SinkType::CsvFile => "csv",
                        SinkType::ProtobufFile => "bin",
                        _ => "txt",
                    };
                    let base_file_name = format!(
                        "{}/vehicles-info-{}.{}",
                        dir,
                        chrono::Utc::now().format("%Y-%m-%d"),
                        extension
                    );
                    let details_file_name = format!(
                        "{}/details-info-{}.{}",
                        dir,
                        chrono::Utc::now().format("%Y-%m-%d"),
                        extension
                    );
                    let prices_file_name = format!(
                        "{}/prices-info-{}.{}",
                        dir,
                        chrono::Utc::now().format("%Y-%m-%d"),
                        extension
                    );
                    create_file_if_not_exists(&base_file_name.as_str(), Some(VEHICLE_HEADER));
                    create_file_if_not_exists(&&details_file_name.as_str(), Some(DETAILS_HEADER));
                    create_file_if_not_exists(&&prices_file_name.as_str(), Some(PRICES_HEADER));

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
                    if let Err(e) = std::fs::create_dir_all(&dir) {
                        error!("Failed to create directory {}: {}", dir, e);
                        // Handle the error appropriately, e.g., return an error, exit with a non-zero code, etc.
                        return; // Or another suitable error handling mechanism
                    }
                    info!("Data directory: {}", dir);
                    // Create the file name with the current date
                    //let file_name = format!("{}/base-info-{}.csv", dir, CREATED_ON);
                    let extension = match sink_type {
                        SinkType::CsvFile => "csv",
                        SinkType::ProtobufFile => "bin",
                        _ => "txt",
                    };
                    let base_file_name = format!(
                        "{}/vehicles-info-{}.{}",
                        dir,
                        chrono::Utc::now().format("%Y-%m-%d"),
                        extension
                    );
                    let details_file_name = format!(
                        "{}/details-info-{}.{}",
                        dir,
                        chrono::Utc::now().format("%Y-%m-%d"),
                        extension
                    );
                    let prices_file_name = format!(
                        "{}/prices-info-{}.{}",
                        dir,
                        chrono::Utc::now().format("%Y-%m-%d"),
                        extension
                    );
                    create_file_if_not_exists(&base_file_name.as_str(), Some(VEHICLE_HEADER));
                    create_file_if_not_exists(&&details_file_name.as_str(), Some(DETAILS_HEADER));
                    create_file_if_not_exists(&&prices_file_name.as_str(), Some(PRICES_HEADER));

                    // Check if the file exists
                }
            }
            info!("Puppeteer command is not implemented yet");
            run_consumers(broker(), sink_type).await;
        }
    }
}

fn create_file_if_not_exists(file_name: &str, header: Option<&str>) {
    // Check if the file exists
    if Path::new(file_name).exists() {
        info!("File {} already exists", file_name);
    } else {
        // Create the file if it doesn't exist
        let _file = File::create(file_name).expect("Failed to create file");
        // Optionally, write the header to the file
        if let Some(header) = header {
            use std::io::Write;
            let mut file = File::options()
                .append(true)
                .create(true)
                .open(file_name)
                .expect("Failed to open file");
            writeln!(file, "{}", header).expect("Failed to write header");
        }
        info!("File {} created", file_name);
    }
}

async fn run_crawler(crawler: String, threads: usize, sink_type: SinkType) {
    let new_group = Uuid::new_v4().to_string();
    let statuses = processMessages(&broker(), &new_group, "status_info", 15).await;
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
        info!("Starting mobile.bg with #{} searches", searches.len());
        log_and_search(searches, crawler, sink_type).await;
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
        log_and_search(searches, crawler, sink_type).await;
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
        log_and_search(searches, crawler, sink_type).await;
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
        log_and_search(searches, crawler, sink_type).await;
    } else if crawler == CRAWLER_AUTOUNCLE_PL {
        let filter = if let Some(found) = map.get(&crawler) {
            found.to_vec()
        } else {
            vec![]
        };
        let searches = filter_searches(&crawler, filter);
        let crawler = AutounclePLScraper::new(AUTOUNCLE_PL_URL, 250);
        let searches = searches.chunks(threads);
        info!("Starting autouncle.pl with #{} searches", searches.len());
        log_and_search(searches, crawler, sink_type).await;
    } else if crawler == CRAWLER_AUTOUNCLE_CH {
        let filter = if let Some(found) = map.get(&crawler) {
            found.to_vec()
        } else {
            vec![]
        };
        let searches = filter_searches(&crawler, filter);
        let crawler = AutouncleCHScraper::new(AUTOUNCLE_CH_URL, 250);
        let searches = searches.chunks(threads);
        info!("Starting autouncle.pl with #{} searches", searches.len());
        log_and_search(searches, crawler, sink_type).await;
    } else if crawler == CRAWLER_AUTOUNCLE_DE {
        let filter = if let Some(found) = map.get(&crawler) {
            found.to_vec()
        } else {
            vec![]
        };
        let searches = filter_searches(&crawler, filter);
        let crawler = AutouncleCHScraper::new(AUTOUNCLE_DE_URL, 250);
        let searches = searches.chunks(threads);
        info!("Starting autouncle.pl with #{} searches", searches.len());
        log_and_search(searches, crawler, sink_type).await;
    } else if crawler == CRAWLER_AUTOUNCLE_IT {
        let filter = if let Some(found) = map.get(&crawler) {
            found.to_vec()
        } else {
            vec![]
        };
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
        let group = uuid::Uuid::new_v4().to_string();
        consumeMobileDeJsons(&broker, &group, MOBILE_DE_TOPIC, sink_type).await
    });
    let r1 = tokio::spawn(task).await;
    if r1.is_ok() {
        info!("car.gr consumer finished");
    } else {
        info!("car.gr consumer failed");
    }
}
