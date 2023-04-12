use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::Read;

use log::debug;

use super::model::VehiclePrice;

fn load_data_into_hashmap(
    file_path: &str,
) -> Result<HashMap<String, VehiclePrice>, Box<dyn Error>> {
    let mut file = File::open(file_path)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;

    let mut reader = csv::Reader::from_reader(data.as_bytes());
    let mut hashmap = HashMap::new();
    for record in reader.deserialize() {
        if record.is_err() {
            continue;
        }
        let record: VehiclePrice = record?;
        println!("{:?}", record);
        hashmap.insert(record.id.clone(), record);
    }

    Ok(hashmap)
}

pub struct DataProcessor {
    file_name: String,
    ids: HashSet<String>,
    data: HashMap<String, VehiclePrice>,
    new_data: HashMap<String, VehiclePrice>,
}

impl DataProcessor {
    pub fn from_file(file_name: &str) -> Result<Self, Box<dyn Error>> {
        let path = std::path::Path::new(file_name);
        if !path.exists() {
            File::create(file_name)?;
            debug!("File {} created.", file_name);
            return Ok(DataProcessor {
                file_name: file_name.to_string(),
                ids: HashSet::new(),
                data: HashMap::new(),
                new_data: HashMap::new(),
            });
        }
        let data = load_data_into_hashmap(file_name)?;
        let ids = data.keys().cloned().collect();
        Ok(DataProcessor {
            file_name: file_name.to_string(),
            ids,
            data,
            new_data: HashMap::new(),
        })
    }

    pub fn process(&mut self, source: &Vec<VehiclePrice>) {
        for value in source {
            self.new_data.insert(value.id.clone(), value.clone());
        }
        
        if self.data.is_empty() {
            return;
        }

        for value in source {
            self.new_data.insert(value.id.clone(), value.clone());
        }

        let source_ids = self.new_data.keys().cloned().collect::<HashSet<String>>();
        let removed_ids = &self.ids - &source_ids;
        for sold_id in removed_ids {
            let mut sold = self.data.get(&sold_id).unwrap().clone();
            sold.sold = true;
            self.new_data.insert(sold_id, sold);
        }
    }

    pub fn save(&mut self) -> Result<(), Box<dyn Error>> {
        let mut writer = csv::Writer::from_path(&self.file_name)?;
        for (_, value) in &self.new_data {
            writer.serialize(value)?;
        }
        self.data = self.new_data.clone();
        self.ids = self.data.keys().cloned().collect();
        self.new_data.clear();
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use log::info;

    use crate::{
        configure_log4rs,
        mobile_scraper::{get_vehicles_prices, utils::read_file_from},
    };

    use super::*;

    #[test]
    fn test_load_data_into_hashmap() {
        configure_log4rs();
        info!("test_load_data_into_hashmap");
        let mut mercedes_processor =
            DataProcessor::from_file("resources/csv/Mercedes_SL.csv").unwrap();
        let html = read_file_from("resources/html", "Mercedes_SL.html").unwrap();
        let vehicle_prices: Vec<VehiclePrice> = get_vehicles_prices(&html);
        mercedes_processor.process(&vehicle_prices);
        assert_eq!(mercedes_processor.data.len(), 0);
        mercedes_processor.save().unwrap();
        assert_eq!(mercedes_processor.data.len(), 11);
        let mut new_mercedes_sl = vec![];
        for i in 1..11 {
            if i % 3 == 0 {
                let mut m = vehicle_prices[i].clone();
                m.id = format!("{}-{}", i, i);
                new_mercedes_sl.push(m);
            } else if i % 4 == 0 {
                let mut m = vehicle_prices[i].clone();
                m.sold = true;
                new_mercedes_sl.push(m);
            } else {
                new_mercedes_sl.push(vehicle_prices[i].clone());
            }
        }
        assert_eq!(new_mercedes_sl.len(), 10);
        mercedes_processor.process(&new_mercedes_sl);
        mercedes_processor.save().unwrap();
        assert_eq!(mercedes_processor.data.len(), 14);
        let mut solded = 0;
        for m in mercedes_processor.data.values() {
            if m.sold {
                solded += 1;
            }
        }
        //Deleted are 6 = 2 (%4) + 3 (%3) + 1 (i == 0)
        assert_eq!(solded, 6);
        mercedes_processor.data.clear();
        mercedes_processor.process(&vehicle_prices);
        mercedes_processor.save().unwrap();
        assert_eq!(mercedes_processor.data.len(), 11);
    }
}
