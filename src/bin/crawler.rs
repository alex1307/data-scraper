use data_scraper::model::Search::Search;
use std::fmt::Debug;
use std::sync::Arc;

use data_scraper::constants::URL::{
    AUTOUNCLE_CH_URL, AUTOUNCLE_DE_URL, AUTOUNCLE_FR_URL, AUTOUNCLE_IT_URL, AUTOUNCLE_NL_URL,
    AUTOUNCLE_PL_URL, AUTOUNCLE_RO_URL, MOBILE_BG_URL,
};

use data_scraper::LOG_CONFIG;
use data_scraper::model::VehicleDataModel::DownloadStatus;
use data_scraper::scraper::AutouncleScraper;
use data_scraper::scraper::BrowserController::BrowserController;

use data_scraper::scraper::VehicleTraits::VehicleScrapeTrait;
use data_scraper::services::AutouncleFilterService;
use data_scraper::services::ScraperAppVehicleService;
use data_scraper::services::SearchBuilder::{
    CRAWLER_AUTOUNCLE_CH, CRAWLER_AUTOUNCLE_DE, CRAWLER_AUTOUNCLE_IT, CRAWLER_AUTOUNCLE_PL,
    ID_AUTOUNCLE_CH_START, ID_AUTOUNCLE_DE_START, ID_AUTOUNCLE_FR, ID_AUTOUNCLE_IT_START,
    ID_AUTOUNCLE_NL_START, ID_AUTOUNCLE_PL_START, ID_AUTOUNCLE_RO_START, ID_MOBILE_BG_START,
    build_autouncle_searches,
};
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

use clap::{Parser, command};

pub const CHUNK_SIZE: usize = 4;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "autouncle.ro")]
    source: String,
    #[arg(short = 'c', long, default_value_t = false)]
    use_chrome: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    configure_log4rs(&LOG_CONFIG);
    dotenvy::dotenv().ok();
    info!("Starting data scraper ==1==...");
    let args = Cli::parse();
    let source = args.source.clone();
    let use_chrome = args.use_chrome;

    let sink_type = SinkType::PostgresDB; // default; can be made configurable later per-YAML

    run_vehicle_crawler(source, 1, sink_type, use_chrome).await;
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

fn source_to_config_path(source: &str) -> Option<String> {
    if let Some(rest) = source.strip_prefix("autouncle.") {
        let market = rest.trim();
        if !market.is_empty() {
            return Some(format!("config/autouncle/{}", market));
        }
    }
    None
}

fn filter_searches(source: &str, filter: Vec<DownloadStatus>) -> Vec<Search> {
    // 1) New path: if source matches autouncle.* use the YAML-driven AutouncleFilterService
    if let Some(cfg_path) = source_to_config_path(source) {
        let searches = AutouncleFilterService::build_searches(&cfg_path);
        info!(
            "Converting {} (via YAML {}) to #{} searches",
            source,
            cfg_path,
            searches.len()
        );
        let mut converted: Vec<Search> = searches.clone();
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
        return converted;
    }

    // 2) Legacy path (kept for mobile.bg until it is migrated)
    let searches = match source {
        CRAWLER_MOBILE_BG => build_mobile_bg_all_searches(MOBILE_BG_URL, ID_MOBILE_BG_START),
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
