use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::{collections::HashSet, fmt::Debug};

use log::{error, info};

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::model::traits::{Header, Identity};
use crate::writer::persistance::{MobileData, MobileDataWriter};

fn load_data<T: Clone + DeserializeOwned + Debug>(file_path: &str) -> Vec<T> {
    let mut file = File::open(file_path).unwrap();
    let mut data = String::new();
    match file.read_to_string(&mut data) {
        Ok(_) => {}
        Err(e) => {
            error!("Error while reading file: {:?}", e);
            return vec![];
        }
    }

    let mut reader = csv::Reader::from_reader(data.as_bytes());
    let mut values = vec![];
    for record in reader.deserialize() {
        if record.is_err() {
            error!("Error while reading record: {:?}", record);
            continue;
        }
        let record: T = record.unwrap();
        values.push(record);
    }
    values
}

#[derive(Debug, Clone)]
pub struct DataProcessor<T: Identity + Clone + Header> {
    files: Vec<String>,
    ids: HashSet<String>,
    updated_ids: HashSet<String>,
    values: Vec<T>,
    do_update: bool,
}

impl<T: Identity + Clone + Header + Debug + DeserializeOwned + Serialize> DataProcessor<T> {
    pub fn from_files(files: Vec<&str>) -> Self {
        let mut values = vec![];
        let mut ids = HashSet::new();
        for file_name in files.clone() {
            let path = Path::new(file_name);
            if !path.exists() {
                error!("File {} does not exist", file_name);
                continue;
            }
            let mut file_values = load_data(file_name);
            info!("Found {} records in file {}", file_values.len(), &file_name);
            ids.extend(file_values.iter().map(|v: &T| v.get_id()));
            values.append(&mut file_values);
        }
        info!("Unique ids: {}", ids.len());
        DataProcessor {
            files: files.iter().map(|f| f.to_string()).collect(),
            ids,
            updated_ids: HashSet::new(),
            values,
            do_update: false,
        }
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
        let target_file_name = target.unwrap_or(&self.files[0]);
        data.write_csv(target_file_name, false).unwrap();

        self.values.append(&mut new_values.clone());
        self.ids.extend(new_values.iter().map(|v| v.get_id()));
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

    pub fn get_values(&self) -> Vec<T> {
        self.values.clone()
    }

    pub fn extend_ids(&mut self, ids: HashSet<String>) -> &Self {
        self.ids.extend(ids);
        self
    }
}

#[cfg(test)]
mod test {}
