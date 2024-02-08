use prost::Message;
use rdkafka::config::ClientConfig;
use rdkafka::message::{Header, OwnedHeaders};
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

pub(crate) fn create_producer(brokers: &str) -> FutureProducer {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .create()
        .expect("Producer creation error")
}

pub async fn send_message(producer: &FutureProducer, topic: &str, message: Vec<u8>) {
    let headers = OwnedHeaders::new().insert(Header {
        key: "command",
        value: Some("process".as_bytes()),
    });

    let record = FutureRecord::to(topic)
        .headers(headers)
        .payload(&message)
        .key("some_key"); // Optional key

    match producer.send(record, Duration::from_secs(0)).await {
        Ok(delivery) => println!("Sent message to {:?}", delivery),
        Err((e, _)) => println!("Error sending message: {:?}", e),
    }
}

pub fn encode_message<T: Message>(message: &T) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    message
        .encode(&mut buf)
        .map(|_| buf)
        .map_err(|e| format!("Error encoding message: {:?}", e))
}

#[cfg(test)]
mod kafka_tests {

    use crate::{
        kafka::KafkaProducer::{create_producer, encode_message, send_message},
        protos::vehicle_model::BaseVehicleInfo,
    };

    #[tokio::test]
    async fn test_send_message() {
        let producer = create_producer("localhost:9094");

        for i in 0..1000 {
            let message = BaseVehicleInfo {
                id: i.to_string(),
                source: "mobile.bg".to_string(),
                make: "BMW".to_string(),
                model: "X5".to_string(),
                title: "BMW X5 3.0d".to_string(),
                currency: "EUR".to_string(),
                price: 10000 + i * 10,
                millage: 100000 + i * 100,
                month: 1,
                year: 2010,
                engine: "Diesel".to_string(),
                gearbox: "Automatic".to_string(),
                cc: 3000,
                power_ps: 300,
                power_kw: 250,
            };
            let encoded_message = encode_message(&message).unwrap();
            send_message(&producer, "base_info", encoded_message).await;
        }
    }
}
