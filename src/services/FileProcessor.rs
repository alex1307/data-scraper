use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::{collections::HashSet, fmt::Debug};

use log::{error, info};

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::model::traits::{Header, Identity};
use crate::writer::DataPersistance::{MobileData, MobileDataWriter};

fn load_data<T: Clone + DeserializeOwned + Debug>(
    file_path: &str,
) -> Result<Vec<T>, Box<dyn Error>> {
    let mut file = File::open(file_path)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;

    let mut reader = csv::Reader::from_reader(data.as_bytes());
    let mut values = vec![];
    for record in reader.deserialize() {
        if record.is_err() {
            error!("Error while reading record: {:?}", record);
            continue;
        }
        let record: T = record?;
        values.push(record);
    }
    Ok(values)
}
#[derive(Debug, Clone)]
pub struct DataProcessor<T: Identity + Clone + Header> {
    file_name: String,
    ids: HashSet<String>,
    updated_ids: HashSet<String>,
    values: Vec<T>,
    do_update: bool,
}

impl<T: Identity + Clone + Header + Debug + DeserializeOwned + Serialize> DataProcessor<T> {
    pub fn from_file(file_name: &str) -> Result<Self, Box<dyn Error>> {
        let values = load_data(&file_name)?;
        info!("Found {} records in file {}", values.len(), &file_name);
        let ids: HashSet<String> = values.iter().map(|v: &T| v.get_id().clone()).collect();
        info!("Unique ids: {}", ids.len());
        Ok(DataProcessor {
            file_name: file_name.to_string(),
            ids,
            updated_ids: HashSet::new(),
            values,
            do_update: false,
        })
    }

    pub fn new_values(&self, source: &Vec<T>) -> Vec<T> {
        if source.is_empty() {
            return vec![];
        }
        let mut new_values = vec![];
        source.iter().for_each(|v| {
            if !self.ids.contains(&v.get_id()) {
                new_values.push(v.clone());
            }
        });
        new_values
    }

    pub fn process(&mut self, source: &Vec<T>, target: Option<&str>) -> Vec<T> {
        if source.is_empty() {
            return vec![];
        }

        //Get new and updated values
        let mut new_values = vec![];
        let mut updated_values = vec![];
        source.iter().for_each(|v| {
            if !self.ids.contains(&v.get_id()) {
                new_values.push(v.clone());
            } else {
                updated_values.push(v.clone());
            }
        });
        info!("Found new values: {}", new_values.len());
        //Save the new values only
        let data = MobileData::Payload(new_values.clone());
        let target_file_name = target.unwrap_or(&self.file_name);
        data.write_csv(target_file_name, false).unwrap();

        self.values.append(&mut new_values.clone());
        self.ids
            .extend(new_values.iter().map(|v| v.get_id().clone()));
        if self.do_update {
            for value in updated_values {
                self.values
                    .iter_mut()
                    .filter(|v| v.get_id() == value.get_id())
                    .for_each(|v| *v = value.clone());
                self.updated_ids.insert(value.get_id());
            }
        }
        new_values
    }

    pub fn do_update(&mut self, do_update: bool) {
        self.do_update = do_update;
    }

    pub fn get_updated_ids(&self) -> &HashSet<String> {
        &self.updated_ids
    }

    pub fn get_ids(&self) -> &HashSet<String> {
        &self.ids
    }

    pub fn get_values(&self) -> &Vec<T> {
        &self.values
    }
}

#[cfg(test)]
mod test {

    use std::fs::remove_file;

    use log::info;

    use crate::{
        configure_log4rs,
        downloader::{Scraper::get_vehicles_prices, Utils::read_file_from},
        model::list::MobileList,
    };

    use super::*;

    #[test]
    fn test_load_data_into_hashmap() {
        configure_log4rs();
        info!("test_load_data_into_hashmap");
        let test_file = "resources/test-data/csv/test_data.csv";
        std::fs::copy("resources/test-data/csv/source.csv", test_file).unwrap();
        let mut mercedes_processor: DataProcessor<MobileList> =
            DataProcessor::from_file(test_file).unwrap();
        let html = read_file_from("resources/html", "Mercedes_SL.html").unwrap();
        let vehicle_prices: Vec<MobileList> = get_vehicles_prices(&html);
        assert_eq!(mercedes_processor.values.len(), 10);
        assert_eq!(vehicle_prices.len(), 11);
        mercedes_processor.process(&vehicle_prices, None);
        assert_eq!(mercedes_processor.values.len(), 11);
        remove_file(test_file).unwrap();
    }
}
