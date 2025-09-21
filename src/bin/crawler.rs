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
use data_scraper::services::ScraperAppVehicleService::{JobError, JobResult, JobStatus, run_job};
use data_scraper::services::SearchBuilder::{
    CRAWLER_AUTOUNCLE_CH, CRAWLER_AUTOUNCLE_DE, CRAWLER_AUTOUNCLE_IT, CRAWLER_AUTOUNCLE_PL,
    ID_AUTOUNCLE_CH_START, ID_AUTOUNCLE_DE_START, ID_AUTOUNCLE_FR, ID_AUTOUNCLE_IT_START,
    ID_AUTOUNCLE_NL_START, ID_AUTOUNCLE_PL_START, ID_AUTOUNCLE_RO_START, build_autouncle_searches,
};
use data_scraper::slack::SlackNotifier::{Channel, SlackNotifier};
use data_scraper::writer::sink::SinkType; // Ensure SinkType includes the Protobuf variant or adjust accordingly
use data_scraper::{
    scraper::MobileBgScraper::MobileBGScraper,
    services::SearchBuilder::{
        CRAWLER_AUTOUNCLE_FR, CRAWLER_AUTOUNCLE_NL, CRAWLER_AUTOUNCLE_RO, CRAWLER_MOBILE_BG,
    },
    utils::helpers::configure_log4rs,
};

use log::{error, info};

use clap::{Parser, command};

use data_scraper::services::SearchBuilder::{CRAWLER_KEY, ID_KEY};
use std::ffi::OsStr;
use walkdir::WalkDir;

use data_scraper::utils::ConfigLoader::{ScraperConfig, load_config};
use std::collections::HashMap;

use gethostname::gethostname;

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
    let notifier = SlackNotifier::from_env();

    let browser = if use_chrome {
        Some(Arc::new(BrowserController::new().await.unwrap()))
    } else {
        None
    };
    let filter = vec![];
    if crawler == CRAWLER_MOBILE_BG {
        let searches = filter_searches(&crawler, filter);

        let searches_vec = searches; // already built by filter_searches
        let scraper = MobileBGScraper::new(MOBILE_BG_URL, 250);
        let main_url = searches_vec.first().map(|s| s.url.clone());
        info!("Starting mobile.bg with {} searches", searches_vec.len());
        let host = gethostname().to_string_lossy().into_owned();
        match run_job(
            scraper,
            searches_vec,
            sink_type.clone(),
            browser.clone(),
            CRAWLER_MOBILE_BG.to_string(),
            Some("config/mobile.bg".to_string()),
            main_url,
        )
        .await
        {
            Ok(status) => {
                notifier
                    .notify_job_finished_with_url(
                        CRAWLER_MOBILE_BG,
                        status.cfg_path.as_deref(),
                        status.actual as i32,
                        status.duration_s,
                        &host,
                        status.url.as_deref(),
                    )
                    .await;
            }
            Err(err) => {
                let mut msg = format!(
                    "🛑 {} failed\ncfg={}\nerror={}\nhost={}",
                    CRAWLER_MOBILE_BG,
                    err.cfg_path.as_deref().unwrap_or("-"),
                    err.message,
                    host
                );
                if let Some(u) = err.url.as_deref() {
                    msg.push_str(&format!("\nurl={}", u));
                }
                notifier.notify_text(Channel::Error, &msg, None).await;
            }
        }
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
                notifier
                    .notify_text(
                        Channel::Error,
                        &format!("🛑 Invalid crawler requested: `{}`", crawler),
                        None,
                    )
                    .await;
                return;
            }
        };
        let searches_vec = filter_searches(&crawler, filter);
        let scraper = AutouncleScraper::AutouncleScraper::new(url, "page", &crawler, 250);
        let main_url = searches_vec.first().map(|s| s.url.clone());
        let host = gethostname().to_string_lossy().into_owned();
        match run_job(
            scraper.clone(),
            searches_vec,
            sink_type.clone(),
            browser.clone(),
            crawler.clone(),
            source_to_config_path(&crawler),
            main_url,
        )
        .await
        {
            Ok(status) => {
                notifier
                    .notify_job_finished_with_url(
                        &crawler,
                        status.cfg_path.as_deref(),
                        status.actual as i32,
                        status.duration_s,
                        &host,
                        status.url.as_deref(),
                    )
                    .await;
            }
            Err(err) => {
                let mut msg = format!(
                    "🛑 {} failed\ncfg={}\nerror={}\nhost={}",
                    &crawler,
                    err.cfg_path.as_deref().unwrap_or("-"),
                    err.message,
                    host
                );
                if let Some(u) = err.url.as_deref() {
                    msg.push_str(&format!("\nurl={}", u));
                }
                notifier.notify_text(Channel::Error, &msg, None).await;
            }
        }
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

fn find_mobilebg_configs(root: &str) -> Vec<String> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension() == Some(OsStr::new("yml")))
        .map(|e| e.path().to_string_lossy().to_string())
        .collect()
}

fn apply_vars(s: &str, vars: &HashMap<String, String>) -> String {
    let mut out = s.to_string();
    for (k, v) in vars {
        out = out.replace(&format!("{{{}}}", k), v);
    }
    out
}

fn build_mobilebg_base_url(cfg: &ScraperConfig) -> String {
    // base + path segments with variables
    let mut base = cfg.base_url.clone();
    let vars: HashMap<String, String> = cfg.variables.clone().unwrap_or_default();
    if let Some(segs) = &cfg.path_segments {
        for seg in segs {
            let filled = apply_vars(seg, &vars);
            if !filled.is_empty() {
                base.push('/');
                base.push_str(&filled);
            }
        }
    }
    // append placeholder for paging (leave {page} unresolved)
    if let Some(t) = &cfg.paging.r#type {
        if t == "path" {
            let pattern = cfg.paging.pattern.as_deref().unwrap_or("p-{page}");
            base.push('/');
            base.push_str(pattern);
        }
    }
    // query params: defaults + filters; keep `extri` raw (~)
    let mut pairs: Vec<(String, String)> = Vec::new();
    if let Some(defs) = &cfg.query_defaults {
        for (k, v) in defs {
            pairs.push((k.clone(), v.clone()));
        }
    }
    for (k, v) in &cfg.filters {
        pairs.push((k.clone(), v.clone()));
    }
    let mut query_parts: Vec<String> = Vec::new();
    for (k, v) in &pairs {
        if k == "extri" {
            query_parts.push(format!("{}={}", k, v));
        } else {
            let enc = url::form_urlencoded::Serializer::new(String::new())
                .append_pair(k, v)
                .finish();
            query_parts.push(enc);
        }
    }
    if query_parts.is_empty() {
        base
    } else {
        format!("{}?{}", base, query_parts.join("&"))
    }
}

fn filter_searches(source: &str, filter: Vec<DownloadStatus>) -> Vec<Search> {
    // 0) New path for mobile.bg – YAML-driven configs under config/mobile.bg/
    if source == CRAWLER_MOBILE_BG {
        let cfg_files = find_mobilebg_configs("config/mobile.bg");
        if cfg_files.is_empty() {
            error!("No mobile.bg configs found under config/mobile.bg");
            return vec![];
        }
        let mut searches: Vec<Search> = Vec::new();
        for (i, cfg_path) in cfg_files.iter().enumerate() {
            match load_config(cfg_path) {
                Ok(cfg) => {
                    let url = build_mobilebg_base_url(&cfg); // contains p-{page} placeholder
                    let mut params = std::collections::HashMap::new();
                    params.insert("url".to_string(), url);
                    params.insert(CRAWLER_KEY.to_string(), CRAWLER_MOBILE_BG.to_string());
                    params.insert(ID_KEY.to_string(), format!("bg-{}", i + 1));
                    searches.push(Search::from(params));
                }
                Err(e) => error!("Failed to load {}: {:?}", cfg_path, e),
            }
        }
        info!(
            "mobile.bg YAMLs: {} → searches: {}",
            cfg_files.len(),
            searches.len()
        );

        let mut converted: Vec<Search> = searches.clone();
        for f in filter {
            if let Some(s) = converted
                .iter()
                .find(|x| x.url == f.url || x.hash == f.hash)
            {
                if let Some(idx) = converted.iter().position(|x| x.id == s.id) {
                    converted.remove(idx);
                }
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
