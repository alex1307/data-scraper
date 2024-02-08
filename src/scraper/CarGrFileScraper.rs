use std::fs;

use log::{error, info};

use crate::{
    helpers::MobileDeHelper::parse_html,
    model::{
        MobileDe::SearchItem,
        VehicleDataModel::{self, Price},
    },
    services::ScraperService::save2file,
    utils::helpers::create_empty_csv,
};

pub fn readDataDir(dir: &str) -> Vec<SearchItem> {
    let mut files = Vec::new();
    let mut items = Vec::new();
    for result in fs::read_dir(dir).unwrap() {
        match result {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    files.push(path);
                }
            }
            Err(e) => {
                error!("Error: {:?}", e);
                continue;
            }
        }
    }

    for file_name in files {
        let content = match fs::read_to_string(file_name) {
            Ok(content) => content,
            Err(e) => {
                error!("Error: {:?}", e);
                continue;
            }
        };

        let json = parse_html(&content);
        match json {
            Ok(json) => {
                info!("{:?}", json);
                items.extend(json.search.srp.data.search_result.items);
            }
            Err(e) => error!("Error: {:?}", e),
        }
    }
    let mut prices = Vec::new();
    let mut consumptions = Vec::new();
    let mut bases = Vec::new();

    for item in items.iter() {
        match Price::try_from(item.clone()) {
            Ok(price) => prices.push(price),
            Err(e) => error!("Error: {:?}", e),
        };
        match VehicleDataModel::Consumption::try_from(item.clone()) {
            Ok(consumption) => consumptions.push(consumption),
            Err(e) => {
                error!("Error: {:?}", e);
                continue;
            }
        };
        match VehicleDataModel::BaseVehicleInfo::try_from(item.clone()) {
            Ok(base) => bases.push(base),
            Err(e) => error!("Error: {:?}", e),
        };
    }
    if create_empty_csv::<VehicleDataModel::Price>("resources/test-data/mobile.de/prices.csv")
        .is_err()
    {
        error!(
            "Failed to create file {}",
            "resources/test-data/mobile.de/prices.csv"
        );
    }

    if create_empty_csv::<VehicleDataModel::Consumption>(
        "resources/test-data/mobile.de/consumptions.csv",
    )
    .is_err()
    {
        error!(
            "Failed to create file {}",
            "resources/test-data/mobile.de/consumptions.csv"
        );
    }

    if create_empty_csv::<VehicleDataModel::BaseVehicleInfo>(
        "resources/test-data/mobile.de/base_info.csv",
    )
    .is_err()
    {
        error!(
            "Failed to create file {}",
            "resources/test-data/mobile.de/base_info.csv"
        );
    }

    save2file("resources/test-data/mobile.de/prices.csv", prices);
    save2file(
        "resources/test-data/mobile.de/consumptions.csv",
        consumptions,
    );
    save2file("resources/test-data/mobile.de/base_info.csv", bases);
    items
}

#[cfg(test)]
mod tests {
    use crate::{utils::helpers::configure_log4rs, LOG_CONFIG};

    use super::*;

    #[test]
    fn read_data_dir() {
        configure_log4rs(&LOG_CONFIG);
        let items = readDataDir("resources/test-data/mobile.de");
        info!("Found: {:?}items", items.len());
    }
}
