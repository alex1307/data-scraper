use super::BASE_INFO_TOPIC;

use crate::{
    model::{
        DataConversionError::ConversionError,
        MobileDe::SearchItem,
        MobileDeAdvJson::processMobileDeJson,
        VehicleDataModel::{self, Vehicle},
    },
    ok_or_message,
    protos::vehicle_model::DownloadStatus,
    unwrap_or_message,
    utils::files::vehicle_file_name,
    writer::{
        flle_writer::file::FileWriter,
        kafka_writer::kafka::KafkaProducer,
        sink::{FormatterType, Sink, SinkType},
    },
};
use futures::StreamExt;
use log::{error, info};
use prost::Message;
use std::time::Duration;

use rdkafka::{
    ClientConfig, Message as KafkaMessage,
    consumer::{Consumer, StreamConsumer},
    message::BorrowedMessage,
};
use tokio::time::timeout;

pub async fn processMessages(
    broker: &str,
    group: &str,
    topic: &str,
    seconds: u64,
) -> Vec<VehicleDataModel::DownloadStatus> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", group.to_owned())
        .set("bootstrap.servers", broker.to_string())
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("Consumer creation failed");

    consumer
        .subscribe(&[topic])
        .expect("Can't subscribe to specified topic");
    let mut message_stream = consumer.stream();
    let mut empty_polls = 0; // Counter for empty polls
    let mut statuses = vec![];
    loop {
        match timeout(Duration::from_secs(seconds), message_stream.next()).await {
            Ok(Some(Ok(message))) => {
                let detached = message.detach();
                let binary = detached.payload().unwrap_or(&[]);

                // Process each message
                if let Ok(status) = process_kafka_message(binary) {
                    let download_status = VehicleDataModel::DownloadStatus {
                        id: status.id,
                        source: status.source,
                        url: status.url,
                        listed: status.listed,
                        actual: status.actual,
                        hash: status.hash,
                    };
                    statuses.push(download_status);
                } else if let Err(e) = process_kafka_message(binary) {
                    error!("Error processing message: {}", e);
                }
                empty_polls = 0; // Reset empty poll counter on message receipt
            }
            Ok(Some(Err(e))) => {
                error!("Kafka error: {}", e);
                break;
            }
            Ok(None) => {
                info!("No more messages or consumer has been closed.");
                empty_polls += 1; // Increment empty polls counter
                if empty_polls >= 1 {
                    // Check if we've waited enough polls without messages
                    break;
                }
            }
            Err(_) => {
                // Timeout reached
                info!(
                    "No messages received in {} seconds, stopping consumer.",
                    seconds
                );
                break;
            }
        }
    }

    consumer.unsubscribe();
    statuses
}

fn process_kafka_message(payload: &[u8]) -> Result<DownloadStatus, String> {
    match DownloadStatus::decode(payload) {
        Ok(download_status) => Ok(download_status),
        Err(e) => Err(format!("Error decoding message: {:?}", e)),
    }
}

pub async fn consumeMobileDeJsons(broker: &str, group: &str, topic: &str, sink_type: SinkType) {
    info!("Starting consumer for topic: {}", topic);
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", group.to_owned())
        .set("bootstrap.servers", broker.to_string())
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("Consumer creation failed");

    consumer
        .subscribe(&[topic])
        .expect("Can't subscribe to specified topic");

    let mut message_stream = consumer.stream();
    let mut base_info_counter = 0;
    let mut price_info_counter = 0;
    let mut details_info_counter = 0;

    let vehicle_sink: Box<dyn Sink<Vehicle>> = match sink_type {
        SinkType::Kafka => Box::new(KafkaProducer::new(
            broker,
            "vehicle",
            FormatterType::Protobuf,
        )),
        SinkType::ProtobufFile => Box::new(FileWriter::new(
            &vehicle_file_name(&sink_type),
            FormatterType::Protobuf,
        )),

        SinkType::CsvFile => Box::new(FileWriter::new(
            &vehicle_file_name(&sink_type),
            FormatterType::Csv,
        )),
    };

    while let Some(message) = message_stream.next().await {
        match message {
            Ok(borrowed_message) => {
                let result = handle_mobile_de_json(&borrowed_message);
                match result {
                    Ok(list) => {
                        for item in list {
                            if let Ok(vehicle) = Vehicle::try_from(item.clone()) {
                                vehicle_sink
                                    .write(vehicle)
                                    .await
                                    .expect("Error writing vehicle to sink");
                                base_info_counter += 1;
                            }
                        }
                    }
                    Err(e) => error!("Error processing message: {}", e),
                }
            }
            Err(e) => error!("Kafka error: {}", e),
        };
        if base_info_counter % 100 == 0 {
            info!("Base info: {}", base_info_counter);
        }
        if price_info_counter % 100 == 0 {
            info!("Price info: {}", price_info_counter);
        }
        if details_info_counter % 100 == 0 {
            info!("Base info: {}", details_info_counter);
        }
    }
}

fn handle_mobile_de_json(message: &BorrowedMessage) -> Result<Vec<SearchItem>, ConversionError> {
    let msg = unwrap_or_message!(
        message.payload_view::<str>(),
        "Error decoding message".to_string()
    );
    let json = ok_or_message!(msg, "Error decoding message".to_string());
    processMobileDeJson(json)
}
