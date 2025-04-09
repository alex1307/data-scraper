use csv::WriterBuilder;
use serde::Serialize;

// src/writer/formatters.rs
use crate::model::VehicleDataModel::Vehicle;

use super::encode_message;

pub trait Formatter<T>: Send + Sync {
    fn format(&self, data: &T) -> Result<Vec<u8>, String>;
}
#[derive(Clone)]
pub struct ProtobufFormatter;

impl Formatter<Vehicle> for ProtobufFormatter {
    fn format(&self, data: &Vehicle) -> Result<Vec<u8>, String> {
        let proto_message = crate::protos::vehicle_model::Vehicle::from(data.clone());
        encode_message(&proto_message)
    }
}

pub struct CsvFormatter;

impl<T: Serialize> Formatter<T> for CsvFormatter {
    fn format(&self, data: &T) -> Result<Vec<u8>, String> {
        let mut writer = WriterBuilder::new()
            .delimiter(b';')
            .has_headers(false)
            .from_writer(vec![]);

        writer.serialize(data).map_err(|e| e.to_string())?;
        writer.flush().map_err(|e| e.to_string())?;
        let csv_data = writer.into_inner().map_err(|e| e.to_string())?;
        Ok(csv_data)
    }
}
