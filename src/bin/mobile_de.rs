

use data_scraper::kafka::KafkaConsumer::consumeMobileDeRawHtml;
use data_scraper::kafka::MetaDataConsumer::consumeMetaTopic;
use data_scraper::kafka::broker;
use data_scraper::kafka::{METADATA_TOPIC, MOBILE_DE_TOPIC};

use data_scraper::LOG_CONFIG;

use data_scraper::writer::sink::SinkType; // Ensure SinkType includes the Protobuf variant or adjust accordingly
use data_scraper::utils::helpers::configure_log4rs;

use log::info;

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

        consumeMobileDeRawHtml(&mobile_de_broker, &group, MOBILE_DE_TOPIC, sink_type).await
    });
    let _ = tokio::join!(meta_task, raw_html_task);
    info!("Data scraper finished.");
}
