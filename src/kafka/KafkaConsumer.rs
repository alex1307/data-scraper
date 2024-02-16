use futures::StreamExt;
use log::{error, info};
use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    message::BorrowedMessage,
    ClientConfig, Message,
};

use crate::{
    helpers::CarGrHTMLHelper::process_listed_links,
    kafka::{CONSUPTION_TOPIC, PRICE_TOPIC},
    model::{
        records::MobileRecord,
        MobileDe::{MobileDeResults, SearchItem},
        VehicleDataModel::{self, Price},
    },
    protos,
};

use super::{
    KafkaProducer::{create_producer, encode_message, send_message},
    BASE_INFO_TOPIC, CHANGE_LOG_TOPIC,
};

pub async fn consumeCarGrHtmlPages(broker: &str, group: &str, topic: &str) {
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

    while let Some(message) = message_stream.next().await {
        match message {
            Ok(borrowed_message) => {
                let list = handle_carg_gr_html(&borrowed_message);
                for item in list {
                    let basic_info = VehicleDataModel::BaseVehicleInfo::from(item.clone());
                    let price_info = VehicleDataModel::Price::from(item.clone());
                    let change_log_info =
                        VehicleDataModel::VehicleChangeLogInfo::from(item.clone());

                    let basic_message = protos::vehicle_model::BaseVehicleInfo::from(basic_info);
                    let price_message = protos::vehicle_model::Price::from(price_info);
                    let change_log_message =
                        protos::vehicle_model::VehicleChangeLogInfo::from(change_log_info);
                    let message = encode_message(&basic_message).unwrap();
                    send_message(&producer, BASE_INFO_TOPIC, message).await;
                    let message = encode_message(&price_message).unwrap();
                    send_message(&producer, PRICE_TOPIC, message).await;
                    let message = encode_message(&change_log_message).unwrap();
                    send_message(&producer, CHANGE_LOG_TOPIC, message).await;
                }
            }
            Err(e) => error!("Kafka error: {}", e),
        }
    }
}

pub async fn consumeMobileDeJsons(broker: &str, group: &str, topic: &str) {
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
        info!("Received message!");
        match message {
            Ok(borrowed_message) => {
                let list = handle_mobile_de_json(&borrowed_message);
                info!("Mobile.de search items: {:?}", list.len());
                for item in list {
                    if let Ok(price) = Price::try_from(item.clone()) {
                        let proto_message = protos::vehicle_model::Price::from(price);
                        let message = encode_message(&proto_message).unwrap();
                        send_message(&producer, PRICE_TOPIC, message).await;
                        price_info_counter += 1;
                    };
                    if let Ok(consumption) = VehicleDataModel::Consumption::try_from(item.clone()) {
                        let proto_message = protos::vehicle_model::Consumption::from(consumption);
                        let message = encode_message(&proto_message).unwrap();
                        send_message(&producer, CONSUPTION_TOPIC, message).await;
                        consumption_info_counter += 1;
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

fn handle_carg_gr_html(message: &BorrowedMessage) -> Vec<MobileRecord> {
    match message.payload_view::<str>() {
        Some(Ok(payload)) => {
            let list = process_listed_links(payload);
            return list;
            // Here you can process the message or forward it to another system
        }
        Some(Err(e)) => error!("Error while deserializing message payload: {:?}", e),
        None => info!("Received message with empty payload"),
    }
    vec![]
}

fn handle_mobile_de_json(message: &BorrowedMessage) -> Vec<SearchItem> {
    match message.payload_view::<str>() {
        Some(Ok(json)) => {
            match serde_json::from_str::<MobileDeResults>(json) {
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
