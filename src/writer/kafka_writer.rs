pub mod kafka {
    use async_trait::async_trait;
    use log::error;

    use rdkafka::{
        ClientConfig,
        producer::{FutureProducer, FutureRecord},
    };
    use std::time::Duration;

    use crate::{
        model::VehicleDataModel::{BaseVehicleInfo, DetailedVehicleInfo, Price},
        writer::{
            formatters::{CsvFormatter, Formatter, ProtobufFormatter},
            sink::{FormatterType, Sink},
        },
    };

    pub struct KafkaProducer {
        producer: FutureProducer,
        destination: String,
        base_formatter: Box<dyn Formatter<BaseVehicleInfo> + Send + Sync>,
        details_formatter: Box<dyn Formatter<DetailedVehicleInfo> + Send + Sync>,
        price_formatter: Box<dyn Formatter<Price> + Send + Sync>,
    }

    impl KafkaProducer {
        pub fn new(brokers: &str, destination: &str, formatter_type: FormatterType) -> Self {
            let producer = ClientConfig::new()
                .set("bootstrap.servers", brokers)
                .create()
                .expect("Producer creation error");
            let base_formatter: Box<dyn Formatter<BaseVehicleInfo> + Send + Sync> =
                match formatter_type {
                    FormatterType::Protobuf => Box::new(ProtobufFormatter),
                    FormatterType::Csv => Box::new(CsvFormatter),
                };
            let details_formatter: Box<dyn Formatter<DetailedVehicleInfo> + Send + Sync> =
                match formatter_type {
                    FormatterType::Protobuf => Box::new(ProtobufFormatter),
                    FormatterType::Csv => Box::new(CsvFormatter),
                };
            let price_formatter: Box<dyn Formatter<Price> + Send + Sync> = match formatter_type {
                FormatterType::Protobuf => Box::new(ProtobufFormatter),
                FormatterType::Csv => Box::new(CsvFormatter),
            };
            Self {
                producer,
                destination: destination.to_string(),
                base_formatter,
                details_formatter,
                price_formatter,
            }
        }
    }

    #[async_trait]
    impl Sink<BaseVehicleInfo> for KafkaProducer {
        async fn write(&self, message: BaseVehicleInfo) -> Result<(), String> {
            let formatted_message = self
                .base_formatter
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

    #[async_trait]
    impl Sink<Price> for KafkaProducer {
        async fn write(&self, message: Price) -> Result<(), String> {
            let formatted_message = self
                .price_formatter
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

    #[async_trait]
    impl Sink<DetailedVehicleInfo> for KafkaProducer {
        async fn write(&self, message: DetailedVehicleInfo) -> Result<(), String> {
            let formatted_message = self
                .details_formatter
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
