use std::fmt::Debug;
use std::sync::Arc;

use data_scraper::constants::URL::{
    AUTOUNCLE_CH_URL, AUTOUNCLE_DE_URL, AUTOUNCLE_FR_URL, AUTOUNCLE_IT_URL, AUTOUNCLE_NL_URL,
    AUTOUNCLE_PL_URL, AUTOUNCLE_RO_URL, MOBILE_BG_URL,
};

use data_scraper::kafka::KafkaConsumer::{consumeMobileDeRawHtml, processMessages};
use data_scraper::kafka::MetaDataConsumer::consumeMetaTopic;
use data_scraper::kafka::broker;
use data_scraper::kafka::{METADATA_TOPIC, MOBILE_DE_TOPIC};

use data_scraper::LOG_CONFIG;
use data_scraper::model::Search::Search;
use data_scraper::model::VehicleDataModel::DownloadStatus;
use data_scraper::scraper::AutouncleScraper;
use data_scraper::scraper::BrowserController::BrowserController;
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
use rdkafka::bindings::rd_kafka_metadata_broker;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    configure_log4rs(&LOG_CONFIG);
    dotenvy::dotenv().ok();
    info!("Starting data scraper...");
    let sink_type = SinkType::PostgresDB;
    let broker = broker();
    let rd_kafka_metadata_broker = broker.clone();
    let mobile_de_broker = broker.clone();
    let meta_task = tokio::spawn(async move {
        consumeMetaTopic(
            &rd_kafka_metadata_broker,
            "raptor-meta-group",
            METADATA_TOPIC,
        )
        .await;
    });
    let raw_html_task = tokio::spawn(async move {
        let group = "mobile_de_group1";

        { consumeMobileDeRawHtml(&mobile_de_broker, &group, MOBILE_DE_TOPIC, sink_type).await }
    });
    let _ = tokio::join!(meta_task, raw_html_task);
    info!("Data scraper finished.");
}
