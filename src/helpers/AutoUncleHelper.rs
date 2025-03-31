use std::{collections::HashMap, fmt::Write, vec};

use log::error;
use regex::Regex;
use scraper::{Html, Selector};

use crate::model::{
    AutouncleJsonModel::{
        CarData, DIESEL_ENGINE_REGEX, ELECTRIC_ENGINE_REGEX, HYBRID_ENGINE_REGEX, LPG_ENGINE_REGEX,
        PETROL_ENGINE_REGEX,
    },
    enums::Engine,
};

fn find_json_bounds(content: &str) -> Option<String> {
    // Find the first index of '{'
    let first_index = content.find('{')?;

    // Find the last index of '}'
    let last_index = content.rfind('}')?;

    // Extract the substring between these indices
    if first_index < last_index {
        Some(content[first_index..=last_index].to_string())
    } else {
        None // Handle case where indices are invalid
    }
}
pub fn process_html(content: &str) -> Vec<CarData> {
    let scripts = get_scripts(content, "carId");
    let mut data = vec![];
    for js in scripts {
        let value = match process_js(js) {
            Some(value) => value,
            None => continue,
        };

        data.push(value);
    }
    data
}

fn process_js(js: String) -> Option<CarData> {
    let content = find_json_bounds(&js).unwrap();
    let json = content.replace("\\\"", "\"").replace(r"\\", r"\\");
    let mut value = match serde_json::from_str::<CarData>(&json) {
        Ok(json) => json,
        Err(e) => {
            error!("Failed to deserialize: {:?}", e.to_string());
            return None;
        }
    };
    if DIESEL_ENGINE_REGEX.captures(&content).is_some() {
        value.engine = Engine::Diesel;
    } else if PETROL_ENGINE_REGEX.captures(&content).is_some() {
        value.engine = Engine::Petrol;
    } else if HYBRID_ENGINE_REGEX.captures(&content).is_some() {
        value.engine = Engine::Hybrid;
    } else if ELECTRIC_ENGINE_REGEX.captures(&content).is_some() {
        value.engine = Engine::Electric;
    } else if LPG_ENGINE_REGEX.captures(&content).is_some() {
        value.engine = Engine::LPG;
    } else {
        error!("Unknown engine type");
        return None;
    }
    let re = Regex::new(r"\d+(\.\d+)? L/100km").unwrap();
    if let Some(caps) = re.captures(&json) {
        if let Some(full_match) = caps.get(0) {
            value.fuel_consumption = Some(full_match.as_str().to_string());
        }
    }
    let re = Regex::new(r"\d+(\.\d+)? Kwh/100 km").unwrap();
    if let Some(caps) = re.captures(&json) {
        if let Some(full_match) = caps.get(0) {
            value.fuel_consumption = Some(full_match.as_str().to_string());
        }
    }
    let re = Regex::new(r"(\d+) g CO2/km (c|k)omb").unwrap();
    if let Some(caps) = re.captures(&json) {
        if let Some(full_match) = caps.get(0) {
            value.co2_emission = Some(full_match.as_str().to_string());
        }
    }
    let re = Regex::new(r"≈ \d+ km").unwrap();
    if let Some(caps) = re.captures(&json) {
        if let Some(full_match) = caps.get(0) {
            value.range = Some(full_match.as_str().to_string());
        }
    }
    Some(value)
}

pub fn get_scripts(html: &str, filter: &str) -> Vec<String> {
    let document = Html::parse_document(html);

    let script_selector = Selector::parse("script").unwrap();
    let scripts = document
        .select(&script_selector)
        .map(|script| script.inner_html())
        .collect::<Vec<String>>()
        .into_iter()
        .filter(|s| s.contains(filter))
        .collect();
    scripts
}

pub fn parse_equipment(content: &str, ids: &Vec<String>) -> HashMap<String, Vec<String>> {
    let script_selector = Selector::parse("script").unwrap();
    let mut equipments = HashMap::new();
    let html = Html::parse_document(content);
    let scripts = html
        .select(&script_selector)
        .map(|script| script.inner_html())
        .collect::<Vec<String>>()
        .into_iter()
        .filter(|s| {
            !s.contains("announcedAsNew")
                && s.contains("self.__next_f.push([1,")
                && s.contains("has")
        })
        .collect::<Vec<String>>();

    for id in ids {
        let pattern = format!(r#"\${}:.*?\]"#, id);
        let re = Regex::new(&pattern).unwrap();
        for s in scripts.iter() {
            if s.contains(id) {
                let js = s.replace(r#"\""#, r#"""#);
                let js = js.replace(r#"\n"#, r#"$"#);
                re.find(&js).map(|caps| -> Option<()> {
                    {
                        let matched = &js[caps.start() + id.len() + 2..caps.end()];
                        let mut json_str = String::new();
                        write!(&mut json_str, r#"{{"{}":{}}}"#, id, matched).unwrap();
                        let result =
                            serde_json::from_str::<HashMap<String, Vec<String>>>(&json_str);
                        if let Ok(json) = result {
                            equipments.extend(json);
                        }
                    };
                    Some(())
                });
                break;
            }
        }
    }
    equipments
}

#[cfg(test)]
mod auto_uncle_tests {
    use std::{
        collections::{HashMap, HashSet},
        fs,
        io::{self, Read},
        vec,
    };

    use log::{error, info};

    use crate::{
        LOG_CONFIG,
        model::{
            AutouncleJsonModel::CarData,
            VehicleDataModel::{BaseVehicleInfo, DetailedVehicleInfo, Price},
        },
        protos::{self},
        utils::helpers::configure_log4rs,
    };

    use super::*;

    #[test]
    fn unique_equipments() {
        configure_log4rs(&LOG_CONFIG);
        let v1 = vec![
            "has_4wd",
            "has_pilot",
            "has_climate_control",
            "has_parking",
            "has_isofix",
            "has_gps",
            "has_esp",
            "has_anti_spin",
            "has_aircondition",
            "has_auto_dimming_mirror",
            "has_rain_sensor",
            "has_stop_and_go",
            "has_lane_warning",
            "has_full_leather",
            "has_tow_bar",
            "has_sunroof",
            "has_sport_seats",
            "has_glass_roof",
            "has_xenon",
            "has_sport_package",
            "has_particle_filter",
            "has_headup_display",
        ];
        let mut set: HashSet<&str> = HashSet::from_iter(v1.iter().cloned());
        let v1 = vec![
            "has_full_leather",
            "has_glass_roof",
            "has_parking",
            "has_tow_bar",
            "has_sunroof",
            "has_stop_and_go",
            "has_gps",
            "has_xenon",
            "has_particle_filter",
            "has_climate_control",
            "has_esp",
            "has_4wd",
            "has_rain_sensor",
            "has_pilot",
            "has_lane_warning",
            "has_anti_spin",
            "has_sport_package",
            "has_aircondition",
            "has_isofix",
            "has_auto_dimming_mirror",
            "has_sport_seats",
        ];
        set.extend(v1.iter().cloned());
        let v1 = vec![
            "has_particle_filter",
            "has_pilot",
            "has_aircondition",
            "has_isofix",
            "has_esp",
            "has_4wd",
            "has_climate_control",
            "has_gps",
            "has_xenon",
            "has_parking",
            "has_tow_bar",
            "has_stop_and_go",
            "has_full_leather",
            "has_glass_roof",
            "has_sunroof",
            "has_lane_warning",
            "has_distance_control",
            "has_sport_seats",
            "has_headup_display",
            "has_driver_alert",
            "has_auto_dimming_mirror",
            "has_rain_sensor",
            "has_anti_spin",
        ];
        set.extend(v1.iter().cloned());
        let v1 = vec![
            "has_4wd",
            "has_pilot",
            "has_parking",
            "has_sport_package",
            "has_sport_seats",
            "has_particle_filter",
            "has_climate_control",
            "has_stop_and_go",
            "has_lane_warning",
            "has_isofix",
            "has_gps",
            "has_esp",
            "has_rain_sensor",
            "has_full_leather",
            "has_glass_roof",
            "has_xenon",
            "has_aircondition",
            "has_sunroof",
            "has_headup_display",
            "has_tow_bar",
            "has_distance_control",
        ];
        set.extend(v1.iter().cloned());
        let v1 = vec![
            "has_4wd",
            "has_aircondition",
            "has_pilot",
            "has_climate_control",
            "has_parking",
            "has_tow_bar",
            "has_stop_and_go",
            "has_isofix",
            "has_gps",
            "has_headup_display",
            "has_full_leather",
            "has_driver_alert",
            "has_auto_dimming_mirror",
            "has_glass_roof",
            "has_sunroof",
            "has_lane_warning",
            "has_el_seats",
            "has_particle_filter",
            "has_esp",
            "has_distance_control",
            "has_xenon",
            "has_rain_sensor",
        ];
        set.extend(v1.iter().cloned());
        let v1 = vec![
            "has_particle_filter",
            "has_4wd",
            "has_parking",
            "has_gps",
            "has_pilot",
            "has_climate_control",
            "has_tow_bar",
            "has_lane_warning",
            "has_glass_roof",
            "has_sunroof",
            "has_esp",
            "has_distance_control",
            "has_auto_dimming_mirror",
            "has_headup_display",
            "has_driver_alert",
            "has_sport_package",
            "has_stop_and_go",
            "has_rain_sensor",
            "has_sport_seats",
            "has_full_leather",
            "has_isofix",
            "has_xenon",
        ];
        set.extend(v1.iter().cloned());
        info!("-------------------");
        info!("equipment: {:?}", set);
        info!("-------------------");
    }

    #[test]
    fn test_read_yml_equipment() {
        configure_log4rs(&LOG_CONFIG);
        let path = "config/car-equipment.yml"; // Replace with the path to your YAML file
        match read_and_parse_yaml(path) {
            Ok(data) => {
                // You can now use 'data' which is a HashMap<i32, Vec<String>>
                // Example: print the data
                for (key, values) in data.iter() {
                    info!("Key: {}", key);
                    for value in values {
                        info!("  Value: {:?}", value);
                    }
                }
            }
            Err(e) => error!("Failed to read or parse YAML file: {}", e),
        }
    }
    type EquipmentDetail = HashMap<i32, Vec<String>>;
    type EquipmentMap = HashMap<String, EquipmentDetail>;

    fn read_and_parse_yaml<P: AsRef<std::path::Path>>(path: P) -> Result<EquipmentMap, io::Error> {
        let mut file = fs::File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let data: EquipmentMap =
            serde_yaml::from_str(&contents).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(data)
    }

    #[test]
    fn test_extract_car_id() {
        configure_log4rs(&LOG_CONFIG);
        let content = fs::read_to_string("resources/test-data/autouncle/2025_script.js").unwrap();
        let content = find_json_bounds(&content).unwrap();
        let json = content.replace("\\\"", "\"").replace(r"\\", r"\\"); // Sanitize JSON
        let value = match serde_json::from_str::<CarData>(&json) {
            Ok(json) => json,
            Err(e) => {
                error!("Failed to deserialize: {:?}", e);
                return;
            }
        };
        info!("Value: {:?}", value);
    }

    #[test]
    fn test_process_html() {
        configure_log4rs(&LOG_CONFIG);
        let content = fs::read_to_string("resources/test-data/autouncle/electric-de.html").unwrap();
        let data = process_html(&content);
        assert_eq!(data.len(), 25 as usize);
        info!("Total: {}", data.len());
        for d in data.into_iter() {
            let basic_info = BaseVehicleInfo::from(d.clone());
            let price_info = Price::from(d.clone());
            let details = DetailedVehicleInfo::from(d.clone());

            info!("Basic Info: {:?}", basic_info);
            info!("Price Info: {:?}", price_info);
            info!("Details: {:?}", details);

            info!(
                "{:?}",
                protos::vehicle_model::BaseVehicleInfo::from(basic_info)
            );
        }
    }

    #[test]
    fn test_process_js() {
        configure_log4rs(&LOG_CONFIG);
        let content = fs::read_to_string("resources/test-data/autouncle/electric.js").unwrap();
        info!("Content: {}", content);
        let json = process_js(content);
        assert!(json.is_some());
        let data = json.unwrap();
        info!("JSON: {:?}", data);
        let basic_info = BaseVehicleInfo::from(data.clone());
        let price_info = Price::from(data.clone());
        let details = DetailedVehicleInfo::from(data.clone());

        info!("Basic Info: {:?}", basic_info);
        info!("Price Info: {:?}", price_info);
        info!("Details: {:?}", details);
    }
}
