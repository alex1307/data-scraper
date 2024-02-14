pub mod KafkaConsumer;
pub mod KafkaProducer;

pub static BASE_INFO_TOPIC: &str = "base_info";
pub static DETAILS_TOPIC: &str = "details_info";
pub static PRICE_TOPIC: &str = "price_info";
pub static CONSUPTION_TOPIC: &str = "consumption_info";
pub static IDS_TOPIC: &str = "ids";
pub static CHANGE_LOG_TOPIC: &str = "change_log";
pub static MOBILE_DE_TOPIC: &str = "mobile_de";
pub static CARS_GR_TOPIC: &str = "car_gr";

pub fn broker() -> String {
    match std::env::var("KAFKA_BROKER") {
        Ok(broker) => broker,
        Err(_) => "localhost:9094".to_string(),
    }
}
