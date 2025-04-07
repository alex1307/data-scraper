const VEHICLE_HEADER: &str = "id;source;make;model;title;year;mileage;engine;gearbox;power_ps;power_kw;currency;price;estimated_price;cc;url;location;equipment;seller_name;seller_url;range;consumption_fuel;consumption_kw;co2;days_in_sale;ranges;rating";
use std::{fs::File, sync::OnceLock};

use std::path::Path;

use crate::writer::sink::SinkType;
use log::{error, info};
pub static DATA_DIR: OnceLock<String> = OnceLock::new();

pub fn create_all_files(sink_type: SinkType) {
    let dir = DATA_DIR.get_or_init(|| "data".to_string());
    if let Err(e) = std::fs::create_dir_all(dir) {
        error!("Failed to create directory {}: {}", dir, e);
        // Handle the error appropriately, e.g., return an error, exit with a non-zero code, etc.
        return; // Or another suitable error handling mechanism
    }

    // Create the file name with the current date
    //let file_name = format!("{}/base-info-{}.csv", dir, CREATED_ON);

    let vehicle_file_name = vehicle_file_name(&sink_type);
    info!("Vehicle file name: {}", vehicle_file_name);
    create_file_if_not_exists(vehicle_file_name.as_str(), Some(VEHICLE_HEADER));
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

fn named_file(label: &str, sink_type: &SinkType) -> String {
    let dir = DATA_DIR.get().unwrap();
    let extension = match sink_type {
        SinkType::CsvFile => "csv",
        SinkType::ProtobufFile => "bin",
        _ => "txt",
    };
    format!(
        "{}/{}-{}.{}",
        dir,
        label,
        chrono::Utc::now().format("%Y-%m-%d"),
        extension
    )
}

pub fn vehicle_file_name(sink_type: &SinkType) -> String {
    named_file("vehicles", sink_type)
}
