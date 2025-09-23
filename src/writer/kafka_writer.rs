pub mod kafka {
    use async_trait::async_trait;
    use log::error;

    use rdkafka::{
        ClientConfig,
        producer::{FutureProducer, FutureRecord},
    };
    use std::time::Duration;

    use crate::{
        model::VehicleDataModel::Vehicle,
        writer::{
            formatters::{CsvFormatter, Formatter, ProtobufFormatter},
            sink::{FormatterType, Sink},
        },
    };

    pub struct KafkaProducer {
        producer: FutureProducer,
        destination: String,
        vehicle_formatter: Box<dyn Formatter<Vehicle> + Send + Sync>,
    }

    impl KafkaProducer {
        pub fn new(brokers: &str, destination: &str, formatter_type: FormatterType) -> Self {
            let producer = ClientConfig::new()
                .set("bootstrap.servers", brokers)
                .create()
                .expect("Producer creation error");

            let vehicle_formatter: Box<dyn Formatter<Vehicle> + Send + Sync> = match formatter_type
            {
                FormatterType::Protobuf => Box::new(ProtobufFormatter),
                FormatterType::Csv => Box::new(CsvFormatter),
            };
            Self {
                producer,
                destination: destination.to_string(),
                vehicle_formatter,
            }
        }
    }

    #[async_trait]
    impl Sink<Vehicle> for KafkaProducer {
        async fn write(&self, message: Vehicle) -> Result<(), String> {
            let formatted_message = self
                .vehicle_formatter
                .format(&message)
                .map_err(|e| format!("Error formatting message: {:?}", e))?;
            let record = FutureRecord::to(&self.destination)
                .payload(&formatted_message)
                .key("crawler"); // Optional key

            match self.producer.send(record, Duration::from_secs(0)).await {
                Ok(_delivery) => Ok(()),
                Err((e, _)) => {
                    error!("Error sending message: {:?}", e);
                    Err(format!("Error sending message: {:?}", e))
                }
            }
        }

        async fn flush(&self) -> Result<(), String> {
            Ok(())
        }
    }
}
