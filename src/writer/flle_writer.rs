use async_trait::async_trait;
use log::error;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::writer::formatters::Formatter;

pub mod file {
    use super::*;
    use crate::model::VehicleDataModel::{BaseVehicleInfo, DetailedVehicleInfo, Price, Vehicle};
    use crate::writer::formatters::{CsvFormatter, ProtobufFormatter};
    use crate::writer::sink::{FormatterType, Sink};

    pub struct FileWriter {
        base_formatter: Box<dyn Formatter<BaseVehicleInfo> + Send + Sync>,
        details_formatter: Box<dyn Formatter<DetailedVehicleInfo> + Send + Sync>,
        price_formatter: Box<dyn Formatter<Price> + Send + Sync>,
        vehicle_formatter: Box<dyn Formatter<Vehicle> + Send + Sync>,
        file: Arc<Mutex<BufWriter<File>>>,
    }

    impl FileWriter {
        pub fn new(file_name: &str, formatter_type: FormatterType) -> Self {
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(PathBuf::from(file_name))
                .expect("Failed to open file");

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
            let vehicle_formatter: Box<dyn Formatter<Vehicle> + Send + Sync> = match formatter_type
            {
                FormatterType::Protobuf => Box::new(ProtobufFormatter),
                FormatterType::Csv => Box::new(CsvFormatter),
            };
            FileWriter {
                base_formatter,
                details_formatter,
                price_formatter,
                vehicle_formatter,
                file: Arc::new(Mutex::new(BufWriter::new(file))),
            }
        }
    }

    #[async_trait]
    impl Sink<BaseVehicleInfo> for FileWriter {
        async fn write(&self, message: BaseVehicleInfo) -> Result<(), String> {
            let formatted_message = self
                .base_formatter
                .format(&message)
                .map_err(|e| format!("Error formatting message: {:?}", e))?;
            self.write_to_file(formatted_message).await
        }

        async fn flush(&self) -> Result<(), String> {
            self.flush_file().await
        }
    }

    #[async_trait]
    impl Sink<DetailedVehicleInfo> for FileWriter {
        async fn write(&self, message: DetailedVehicleInfo) -> Result<(), String> {
            let formatted_message = self
                .details_formatter
                .format(&message)
                .map_err(|e| format!("Error formatting message: {:?}", e))?;
            self.write_to_file(formatted_message).await
        }

        async fn flush(&self) -> Result<(), String> {
            self.flush_file().await
        }
    }

    #[async_trait]
    impl Sink<Price> for FileWriter {
        async fn write(&self, message: Price) -> Result<(), String> {
            let formatted_message = self
                .price_formatter
                .format(&message)
                .map_err(|e| format!("Error formatting message: {:?}", e))?;
            self.write_to_file(formatted_message).await
        }

        async fn flush(&self) -> Result<(), String> {
            self.flush_file().await
        }
    }

    #[async_trait]
    impl Sink<Vehicle> for FileWriter {
        async fn write(&self, message: Vehicle) -> Result<(), String> {
            let formatted_message = self
                .vehicle_formatter
                .format(&message)
                .map_err(|e| format!("Error formatting message: {:?}", e))?;
            self.write_to_file(formatted_message).await
        }

        async fn flush(&self) -> Result<(), String> {
            self.flush_file().await
        }
    }

    impl FileWriter {
        async fn write_to_file(&self, item: Vec<u8>) -> Result<(), String> {
            let mut writer = self.file.lock().await;
            writer.write_all(&item).map_err(|e| e.to_string())?;
            if let Err(e) = writer.flush() {
                error!("Error flushing file: {}", e);
                return Err(format!("Error flushing file: {}", e));
            }
            Ok(())
        }

        async fn flush_file(&self) -> Result<(), String> {
            let mut writer = self.file.lock().await;
            writer
                .flush()
                .map_err(|e| format!("Error flushing file: {}", e))?;
            Ok(())
        }
    }
}
