use log::info;

pub mod KafkaConsumer;
pub mod MetaDataConsumer;
// pub mod KafkaProducer;

pub static BASE_INFO_TOPIC: &str = "base_info";
pub static DETAILS_TOPIC: &str = "details_info";
pub static PRICE_TOPIC: &str = "price_info";
pub static VEHICLE_TOPIC: &str = "vehicle_info";
pub static IDS_TOPIC: &str = "ids";
pub static MOBILE_DE_TOPIC: &str = "raptor.mobile_de";
pub static METADATA_TOPIC: &str = "raptor.metadata";
pub static CARS_GR_TOPIC: &str = "car_gr";
pub static EUR_EXCHANGE_RATE_TOPIC: &str = "exchange_rate";

pub fn broker() -> String {
    match std::env::var("KAFKA_BROKER") {
        Ok(broker) => {
            info!("Kafka broker: {}", broker);
            broker
        }
        Err(_) => "localhost:9094".to_string(),
    }
}
