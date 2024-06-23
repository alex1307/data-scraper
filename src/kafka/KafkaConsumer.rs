use super::{
    KafkaProducer::{create_producer, encode_message, send_message},
    BASE_INFO_TOPIC,
};
use crate::{
    kafka::{CONSUPTION_TOPIC, PRICE_TOPIC},
    model::{
        MobileDe::{MobileDeResults, SearchItem},
        VehicleDataModel::{self, Price},
    },
    protos::{self, vehicle_model::DownloadStatus},
};
use futures::StreamExt;
use log::{debug, error, info};
use prost::Message;
use rdkafka::Message as KafkaMessage;
use serde_ignored::Path;
use serde_json::Value;
use std::{fs, time::Duration};

use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    message::BorrowedMessage,
    ClientConfig,
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

pub async fn consumeMobileDeJsons(broker: &str, group: &str, topic: &str) {
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

    let producer = create_producer(broker);

    let mut message_stream = consumer.stream();
    let mut base_info_counter = 0;
    let mut price_info_counter = 0;
    let mut consumption_info_counter = 0;
    while let Some(message) = message_stream.next().await {
        match message {
            Ok(borrowed_message) => {
                let list = handle_mobile_de_json(&borrowed_message);
                for item in list {
                    if let Ok(price) = Price::try_from(item.clone()) {
                        let proto_message = protos::vehicle_model::Price::from(price);
                        let message = encode_message(&proto_message).unwrap();
                        send_message(&producer, PRICE_TOPIC, message).await;
                        price_info_counter += 1;
                    };
                    if let Ok(consumption) = VehicleDataModel::Consumption::try_from(item.clone()) {
                        let proto_message =
                            protos::vehicle_model::Consumption::from(consumption.clone());
                        let message = encode_message(&proto_message).unwrap();
                        send_message(&producer, CONSUPTION_TOPIC, message).await;
                        consumption_info_counter += 1;
                        info!("Consumption: {:?}", consumption);
                    } else if let Err(err) = VehicleDataModel::Consumption::try_from(item.clone()) {
                        error!("Error converting consumption: {:?}", err);
                        info!("Item: {:?}", item);
                    };
                    if let Ok(base) = VehicleDataModel::BaseVehicleInfo::try_from(item.clone()) {
                        let proto_message = protos::vehicle_model::BaseVehicleInfo::from(base);
                        let message = encode_message(&proto_message).unwrap();
                        send_message(&producer, BASE_INFO_TOPIC, message).await;
                        base_info_counter += 1;
                    };
                }
            }
            Err(e) => error!("Kafka error: {}", e),
        }
        if base_info_counter % 100 == 0 {
            info!("Base info: {}", base_info_counter);
        }
        if price_info_counter % 100 == 0 {
            info!("Price info: {}", price_info_counter);
        }
        if consumption_info_counter % 100 == 0 {
            info!("Base info: {}", consumption_info_counter);
        }
    }
}

fn parse_json(json: &str) -> Result<MobileDeResults, serde_json::Error> {
    let mut ignored = Vec::new();
    //let json_value: Value = serde_json::from_str(json)?;
    //let pretty_json = serde_json::to_string_pretty(&json_value)?;

    // Write the pretty JSON string to the file
    //let mut file = File::create("~/Software/prety_json.json").unwrap();
    //file.write_all(pretty_json.as_bytes()).unwrap();

    let deserializer = &mut serde_json::Deserializer::from_str(json);
    let result = serde_ignored::deserialize(deserializer, |path: Path| {
        ignored.push(path.to_string());
    });

    match result {
        Ok(parsed) => {
            info!("Successfully parsed JSON");
            for path in ignored {
                debug!("Ignored field: {}", path);
            }
            Ok(parsed)
        }
        Err(e) => {
            error!("Error parsing JSON: {:?}", e);
            to_pretty_string(json);
            Err(e)
        }
    }
}

fn to_pretty_string(json: &str) {
    let json_value: Value = match serde_json::from_str(json) {
        Ok(json_value) => json_value,
        Err(e) => {
            error!("Error parsing JSON: {:?}", e);
            return;
        }
    };

    let pretty_json = match serde_json::to_string_pretty(&json_value) {
        Ok(pretty_json) => pretty_json,
        Err(e) => {
            error!("Error pretty printing JSON: {:?}", e);
            return;
        }
    };
    //let json_value: Value = serde_json::from_str(json)?;
    //let pretty_json = serde_json::to_string_pretty(&json_value)?;

    // Write the pretty JSON string to the file
    let filename =
        "_log/prety_json_".to_string() + uuid::Uuid::new_v4().to_string().as_str() + ".json";
    match fs::write(filename.clone(), pretty_json.as_bytes()) {
        Ok(_) => debug!(
            "Successfully wrote pretty JSON to file: {}",
            filename.clone()
        ),
        Err(e) => error!("Error writing pretty JSON to file: {:?}", e),
    }
}

fn handle_mobile_de_json(message: &BorrowedMessage) -> Vec<SearchItem> {
    match message.payload_view::<str>() {
        Some(Ok(json)) => {
            match parse_json(json) {
                Ok(json) => {
                    let list = json.search.srp.data.search_result.items;
                    info!("Mobile.de search items: {:?}", list.len());
                    return list;
                }
                Err(e) => error!("Error: {:?}", e),
            }
            // Here you can process the message or forward it to another system
        }
        Some(Err(e)) => info!("Error while deserializing message payload: {:?}", e),
        None => info!("Received message with empty payload"),
    }
    vec![]
}
