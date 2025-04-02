const VEHICLE_HEADER: &str = "id;source;make;model;title;currency;price;mileage;month;year;engine;gearbox;cc;power_ps;power_kw;search_id;url";
const DETAILS_HEADER: &str = "id;source;location;equipment;seller_name;seller_url;range;consumption_fuel;consumption_kw;co2;days_in_sale";
const PRICES_HEADER: &str =
    "id;source;estimated_price;price;currency;save_difference;overpriced_difference;ranges;rating";

use std::fs::File;

use std::path::Path;

use crate::writer::sink::SinkType;
use log::{error, info};

pub fn create_all_files(dir: &str, sink_type: SinkType) {
    if let Err(e) = std::fs::create_dir_all(&dir) {
        error!("Failed to create directory {}: {}", dir, e);
        // Handle the error appropriately, e.g., return an error, exit with a non-zero code, etc.
        return; // Or another suitable error handling mechanism
    }
    info!("Data directory: {}", dir);
    // Create the file name with the current date
    //let file_name = format!("{}/base-info-{}.csv", dir, CREATED_ON);
    let extension = match sink_type {
        SinkType::CsvFile => "csv",
        SinkType::ProtobufFile => "bin",
        _ => "txt",
    };
    let base_file_name = format!(
        "{}/vehicles-info-{}.{}",
        dir,
        chrono::Utc::now().format("%Y-%m-%d"),
        extension
    );
    let details_file_name = format!(
        "{}/details-info-{}.{}",
        dir,
        chrono::Utc::now().format("%Y-%m-%d"),
        extension
    );
    let prices_file_name = format!(
        "{}/prices-info-{}.{}",
        dir,
        chrono::Utc::now().format("%Y-%m-%d"),
        extension
    );
    create_file_if_not_exists(&base_file_name.as_str(), Some(VEHICLE_HEADER));
    create_file_if_not_exists(&&details_file_name.as_str(), Some(DETAILS_HEADER));
    create_file_if_not_exists(&&prices_file_name.as_str(), Some(PRICES_HEADER));
}

fn create_file_if_not_exists(file_name: &str, header: Option<&str>) {
    // Check if the file exists
    if Path::new(file_name).exists() {
        info!("File {} already exists", file_name);
    } else {
        // Create the file if it doesn't exist
        let _file = File::create(file_name).expect("Failed to create file");
        // Optionally, write the header to the file
        if let Some(header) = header {
            use std::io::Write;
            let mut file = File::options()
                .append(true)
                .create(true)
                .open(file_name)
                .expect("Failed to open file");
            writeln!(file, "{}", header).expect("Failed to write header");
        }
        info!("File {} created", file_name);
    }
}
