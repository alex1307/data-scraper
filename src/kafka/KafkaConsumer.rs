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
        db_writer::db::DBWriter,
        flle_writer::file::FileWriter,
        kafka_writer::kafka::KafkaProducer,
        sink::{FormatterType, Sink, SinkType},
    },
};
use futures::StreamExt;
use log::{error, info};
use prost::Message;
use std::io::Read;
use std::time::Duration;

use rdkafka::{
    ClientConfig, Message as KafkaMessage,
    consumer::{Consumer, StreamConsumer},
    message::{BorrowedMessage, Headers},
};
use tokio::time::timeout;

use flate2::read::GzDecoder;
use serde_json::Value;

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

fn header_str(msg: &BorrowedMessage<'_>, key: &str) -> Option<String> {
    let hs = msg.headers()?;
    for i in 0..hs.count() {
        let h = hs.get(i); // Header { key: &str, value: Option<&[u8]> }
        if h.key.eq_ignore_ascii_case(key) {
            if let Some(v) = h.value {
                if let Ok(s) = std::str::from_utf8(v) {
                    return Some(s.to_owned());
                }
            }
        }
    }
    None
}

fn decode_html_payload(msg: &BorrowedMessage<'_>) -> Result<String, String> {
    let payload = msg.payload().ok_or_else(|| "Empty payload".to_string())?;
    let enc = header_str(msg, "content-encoding").unwrap_or_default();
    if enc.eq_ignore_ascii_case("gzip") {
        let mut d = GzDecoder::new(payload);
        let mut out = String::new();
        d.read_to_string(&mut out)
            .map_err(|e| format!("gunzip error: {}", e))?;
        Ok(out)
    } else {
        String::from_utf8(payload.to_vec()).map_err(|e| format!("utf8 error: {}", e))
    }
}

fn is_srp_url(url: &str) -> bool {
    url.contains("/fahrzeuge/search.html")
}

fn normalize_mobilede_html(html: &str) -> String {
    // Mirror legacy: replace thin space/non‑breaking space and strip escaped angle brackets
    html.replace("\\u2009", " ")
        .replace("\\u00A0", "")
        .replace("\\u003E", "")
        .replace("\\u003C", "")
}

/// Extract embedded JSON from mobile.de SRP HTML without external regex deps.
/// Priority 1: <script id="__NEXT_DATA__">{...}</script>
/// Fallback 2: first <script type="application/json">{...}</script>
fn extract_mobilede_srp_json(html: &str) -> Option<String> {
    // 1) Legacy path: window.__INITIAL_STATE__ = { ... } window.__PUBLIC_CONFIG__
    let norm = normalize_mobilede_html(html);
    let start_marker = "window.__INITIAL_STATE__ = ";
    let end_marker = "window.__PUBLIC_CONFIG__";
    if let (Some(s), Some(e)) = (norm.find(start_marker), norm.find(end_marker)) {
        if e > s {
            let from = s + start_marker.len();
            let slice = &norm[from..e];
            // Trim and remove possible trailing ';'
            let json_str = slice.trim().trim_end_matches(';').trim().to_string();
            if !json_str.is_empty() {
                return Some(json_str);
            }
        }
    }

    // 2) Fallback: Next.js/__NEXT_DATA__ or first application/json script
    fn extract_between(s: &str, start_marker: &str, end_marker: &str) -> Option<String> {
        let start = s.find(start_marker)?;
        // find the closing '>' of the opening <script ...>
        let after_tag = s[start..].find('>')? + start + 1;
        let end_rel = s[after_tag..].find(end_marker)?;
        let end = after_tag + end_rel;
        Some(s[after_tag..end].trim().to_string())
    }
    if let Some(json) = extract_between(html, r#"<script id=\"__NEXT_DATA__\""#, "</script>") {
        return Some(json);
    }
    if let Some(json) = extract_between(html, r#"<script type=\"application/json\""#, "</script>") {
        return Some(json);
    }
    None
}

pub async fn consumeMobileDeJsons(broker: &str, group: &str, topic: &str, sink_type: SinkType) {
    info!(
        "Starting consumer for topic: {} and sink type: {:?}",
        topic, sink_type
    );
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

    let vehicle_sink: Box<dyn Sink<Vehicle>> = match sink_type {
        SinkType::Kafka => {
            info!("Using Kafka sink for vehicles");
            Box::new(KafkaProducer::new(
                broker,
                "vehicle",
                FormatterType::Protobuf,
            ))
        }
        SinkType::ProtobufFile => Box::new(FileWriter::new(
            &vehicle_file_name(&sink_type),
            FormatterType::Protobuf,
        )),

        SinkType::CsvFile => Box::new(FileWriter::new(
            &vehicle_file_name(&sink_type),
            FormatterType::Csv,
        )),

        SinkType::PostgresDB => {
            info!("Using PostgresDB sink for vehicles");
            Box::new(DBWriter::new().await)
        }
    };

    while let Some(message) = message_stream.next().await {
        match message {
            Ok(borrowed_message) => {
                let result = handle_mobile_de_json(&borrowed_message);
                match result {
                    Ok(list) => {
                        info!("Processing {} items from message", list.len());
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

/// Consume raw HTML from Kafka (topic `mobile_de`) and hand it off for parsing.
/// This does NOT decode protobuf/JSON; it expects raw HTML (gzip or plain),
/// as produced by the Node Raptor publisher.
pub async fn consumeMobileDeRawHtml(broker: &str, group: &str, topic: &str, sink_type: SinkType) {
    info!("Starting RAW-HTML consumer for topic: {}", topic);

    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", group.to_owned() + "-raw-html")
        .set("bootstrap.servers", broker.to_string())
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "earliest")
        .set("max.poll.interval.ms", "900000")
        .set("session.timeout.ms", "45000")
        .set("fetch.wait.max.ms", "500")
        .set("socket.keepalive.enable", "true")
        .set("heartbeat.interval.ms", "3000")
        .create()
        .expect("Consumer creation failed");

    consumer
        .subscribe(&[topic])
        .expect("Can't subscribe to specified topic");

    let mut stream = consumer.stream();
    let vehicle_sink: Box<dyn Sink<Vehicle>> = match sink_type {
        SinkType::Kafka => {
            info!("Using Kafka sink for vehicles");
            Box::new(KafkaProducer::new(
                broker,
                "vehicle",
                FormatterType::Protobuf,
            ))
        }
        SinkType::ProtobufFile => Box::new(FileWriter::new(
            &vehicle_file_name(&sink_type),
            FormatterType::Protobuf,
        )),

        SinkType::CsvFile => Box::new(FileWriter::new(
            &vehicle_file_name(&sink_type),
            FormatterType::Csv,
        )),

        SinkType::PostgresDB => {
            info!("Using PostgresDB sink for vehicles");
            Box::new(DBWriter::new().await)
        }
    };
    let mut counter = 0;
    while let Some(ev) = stream.next().await {
        match ev {
            Ok(msg) => {
                // Extract meta
                let page_type_s = header_str(&msg, "x-raptor-page-type");
                let url_s = header_str(&msg, "x-raptor-url");
                let source_s = header_str(&msg, "x-raptor-source");
                let enc_s = header_str(&msg, "content-encoding");

                let page_type = page_type_s.as_deref().unwrap_or("");
                let url = url_s.as_deref().unwrap_or("");
                let source = source_s.as_deref().unwrap_or("");
                let enc = enc_s.as_deref().unwrap_or("");

                // Decode payload to UTF-8 HTML
                match decode_html_payload(&msg) {
                    Ok(html) => {
                        info!(
                            "RAW HTML received: source={}, type={}, enc={}, bytes={}, url={}",
                            source,
                            page_type,
                            enc,
                            html.len(),
                            url
                        );

                        // Only process SRP pages
                        if !page_type.eq_ignore_ascii_case("srp") || !is_srp_url(url) {
                            info!(
                                "Skipping non-SRP payload (type='{}', url='{}')",
                                page_type, url
                            );
                            continue;
                        }

                        let json_candidate = extract_mobilede_srp_json(&html);
                        match json_candidate {
                            Some(json_blob) => {
                                info!("SRP JSON candidate length: {} bytes", json_blob.len());
                                // Validate JSON syntax before handing off
                                match serde_json::from_str::<Value>(&json_blob) {
                                    Ok(_) => match processMobileDeJson(json_blob.as_str()) {
                                        Ok(list) => {
                                            info!("Processing {} items from SRP JSON", list.len());
                                            for item in list {
                                                if let Ok(vehicle) = Vehicle::try_from(item.clone())
                                                {
                                                    vehicle_sink
                                                        .write(vehicle)
                                                        .await
                                                        .expect("Error writing vehicle to sink");
                                                    counter += 1;
                                                }
                                            }
                                        }
                                        Err(e) => error!("Error processing SRP JSON: {}", e),
                                    },
                                    Err(e) => error!("Invalid SRP JSON syntax: {}", e),
                                }
                            }
                            None => {
                                error!("No SRP JSON found in HTML — TODO: hand off to HTML parser");
                                // TODO: When HTML parser is ready, call it here with `html`.
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to decode HTML payload: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Kafka error: {}", e);
            }
        }
        if counter % 100 == 0 {
            info!("Counter: {}", counter);
        }
    }

    consumer.unsubscribe();
    info!("RAW-HTML consumer finished for topic: {}", topic);
}
