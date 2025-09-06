use std::fmt::Debug;
use std::sync::Arc;

use data_scraper::constants::URL::{
    AUTOUNCLE_CH_URL, AUTOUNCLE_DE_URL, AUTOUNCLE_FR_URL, AUTOUNCLE_IT_URL, AUTOUNCLE_NL_URL,
    AUTOUNCLE_PL_URL, AUTOUNCLE_RO_URL, MOBILE_BG_URL,
};
#[cfg(feature = "kafka")]
use data_scraper::kafka::KafkaConsumer::{consumeMobileDeRawHtml, processMessages};
#[cfg(feature = "kafka")]
use data_scraper::kafka::MOBILE_DE_TOPIC;
#[cfg(feature = "kafka")]
use data_scraper::kafka::broker;

use data_scraper::LOG_CONFIG;
use data_scraper::model::Search::Search;
use data_scraper::model::VehicleDataModel::DownloadStatus;
use data_scraper::scraper::AutouncleScraper;
use data_scraper::scraper::BrowserController::BrowserController;
#[cfg(feature = "kafka")]
use data_scraper::writer::sink;

use data_scraper::scraper::VehicleTraits::VehicleScrapeTrait;
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

#[tokio::main(flavor = "current_thread")]
async fn main() {
    configure_log4rs(&LOG_CONFIG);
    dotenvy::dotenv().ok();
    info!("Starting data scraper...");
    let sink_type = SinkType::PostgresDB;
    info!("Puppeteer command is not implemented yet");
    #[cfg(feature = "kafka")]
    run_consumers(broker(), sink_type).await;
    #[cfg(not(feature = "kafka"))]
    {
        error!("Kafka feature is not enabled. Cannot run consumer.");
    }
}

#[cfg(feature = "kafka")]
async fn run_consumers(broker: String, sink_type: SinkType) {
    let task = tokio::spawn(async move {
        let group = "mobile_de_group_1";
        #[cfg(feature = "kafka")]
        {
            consumeMobileDeRawHtml(&broker, &group, MOBILE_DE_TOPIC, sink_type).await
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
