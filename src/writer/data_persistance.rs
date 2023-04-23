use std::{
    error::Error,
    fs::File,
    io::{BufWriter, Write},
};

use csv::WriterBuilder;

use serde::{Deserialize, Serialize};

use crate::mobile_scraper::model::Header;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MobileData<T> {
    Payload(Vec<T>),
    // Other data types can be added here
}

pub trait MobileDataWriter<T> {
    fn write_json(&self, file_path: &str) -> std::io::Result<()>;
    fn write_csv(&self, file_path: &str, has_headers: bool) -> std::io::Result<()>;
}

fn open_file(file_path: &str) -> std::io::Result<File> {
    let file = std::fs::OpenOptions::new()
        .create(false)
        .append(true)
        .open(file_path)?;
    Ok(file)
}

impl<T: Serialize + Clone> MobileDataWriter<T> for MobileData<T> {
    fn write_json(&self, file_path: &str) -> std::io::Result<()> {
        let data = match self {
            MobileData::Payload(v) => serde_json::to_string_pretty(v)?,
        };
        let mut file = open_file(file_path)?;
        file.write_all(data.as_bytes())?;
        Ok(())
    }

    fn write_csv(&self, file_path: &str, has_headers: bool) -> std::io::Result<()> {
        let _data = match self {
            MobileData::Payload(v) => {
                let file = open_file(file_path)?;
                let mut wtr = WriterBuilder::new()
                    .has_headers(has_headers)
                    .from_writer(file);
                for vehicle in v {
                    wtr.serialize(vehicle)?;
                }
                wtr.flush()?;
            }
        };
        Ok(())
    }
}

pub fn create_empty_csv<T: Serialize + Header>(file_path: &str) -> Result<(), Box<dyn Error>> {
    let path = std::path::Path::new(file_path);
    if path.exists() {
        return Err(format!("File {} already exists.", file_path).into());
    }
    let line = T::header().join(","); // Convert the vector to a comma-separated string
    let file = File::create(file_path)?; // Create a new file for writing
    let mut writer = BufWriter::new(file);
    writer.write_all(line.as_bytes())?;
    writer.write_all(b"\r\n")?;
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use csv::ReaderBuilder;

    use super::*;
    use std::fs;
    use std::io::Result;
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Vehicle {
        make: String,
        model: String,
        from_year: u16,
        to_year: u16,
    }

    fn read_file_from_resources(filename: &str) -> Result<String> {
        let path = format!("resources/{}", filename);
        fs::read_to_string(path)
    }

    fn count_csv_records(file_path: &str) -> std::io::Result<usize> {
        let file = File::open(file_path)?;
        let mut reader = ReaderBuilder::new().from_reader(file);
        let mut count = 0;

        for result in reader.records() {
            let _ = result?; // Ignore errors
            count += 1;
        }

        Ok(count)
    }

    #[test]
    fn test_write_json() -> std::io::Result<()> {
        let vehicles = vec![
            Vehicle {
                make: "Toyota".to_string(),
                model: "Corolla".to_string(),
                from_year: 2010,
                to_year: 2015,
            },
            Vehicle {
                make: "Honda".to_string(),
                model: "Civic".to_string(),
                from_year: 2012,
                to_year: 2017,
            },
        ];
        let data = MobileData::Payload(vehicles);

        let test_file = "test.json";
        data.write_json(test_file)?;

        let json_data = fs::read_to_string(test_file)?;
        let expected_json_data = read_file_from_resources(test_file)?;
        assert_eq!(json_data.trim(), expected_json_data);
        fs::remove_file(test_file)?;
        Ok(())
    }
    #[test]
    fn test_write_csv() -> std::io::Result<()> {
        let vehicles = vec![
            Vehicle {
                make: "Toyota".to_string(),
                model: "Corolla".to_string(),
                from_year: 2010,
                to_year: 2015,
            },
            Vehicle {
                make: "Honda".to_string(),
                model: "Civic".to_string(),
                from_year: 2012,
                to_year: 2017,
            },
        ];
        let data = MobileData::Payload(vehicles);

        let test_file = "test.csv";
        data.write_csv(test_file, true)?;

        let csv_data = fs::read_to_string(test_file)?;
        let expected_csv_data = read_file_from_resources(test_file)?;
        assert_eq!(csv_data, expected_csv_data);
        fs::remove_file(test_file)?;
        Ok(())
    }

    #[test]
    fn test_append_csv() -> std::io::Result<()> {
        let vehicles = vec![
            Vehicle {
                make: "Toyota".to_string(),
                model: "Corolla".to_string(),
                from_year: 2010,
                to_year: 2015,
            },
            Vehicle {
                make: "Honda".to_string(),
                model: "Civic".to_string(),
                from_year: 2012,
                to_year: 2017,
            },
        ];
        let data = MobileData::Payload(vehicles);

        let test_file = "test.csv";
        data.write_csv(test_file, true)?;

        let csv_data = fs::read_to_string(test_file)?;
        let expected_csv_data = read_file_from_resources(test_file)?;
        assert_eq!(csv_data, expected_csv_data);

        let new_vehicles = vec![
            Vehicle {
                make: "Skoda".to_string(),
                model: "Octavia".to_string(),
                from_year: 2010,
                to_year: 2015,
            },
            Vehicle {
                make: "VW".to_string(),
                model: "Golf".to_string(),
                from_year: 2012,
                to_year: 2017,
            },
        ];
        let data = MobileData::Payload(new_vehicles);
        data.write_csv(test_file, false)?;
        let number_of_records = count_csv_records(test_file)?;
        assert_eq!(number_of_records, 4);
        fs::remove_file(test_file)?;
        Ok(())
    }
}
