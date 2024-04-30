use std::{collections::HashMap, fmt::Write, vec};

use log::error;
use regex::Regex;
use scraper::{Html, Selector};
use serde::Deserialize;

use crate::model::AutoUncleVehicle::{AutoUncleVehicle, Root};
#[derive(Deserialize, Debug)]
struct PaginatedCars {
    #[serde(rename = "carsPaginated")]
    paginated: Cars,
}

#[derive(Deserialize, Debug)]
struct Cars {
    cars: Vec<AutoUncleVehicle>,
}

pub fn list_vehicles_from_text(txt: &str) -> Vec<AutoUncleVehicle> {
    let start = txt.find("carsPaginated").unwrap();
    let end = txt.find("pagination").unwrap();
    let paginated = txt[start - 1..end - 1].to_string();
    let processed = paginated.replace(r#"\""#, r#"""#);
    let processed = processed.replace('\\', "");
    let processed = processed.replace(r#"\\"#, r#"\"#);
    let processed = processed.replace(r#"\n"#, r#""#);
    let processed = processed.replace(r#"\t"#, r#""#);
    let processed = processed.replace(r#"\r"#, r#""#);
    let processed = processed.replace(r#""""#, r#"""#);
    let processed = processed.replace("}],", "}],\n");
    let processed = processed.replace("},", "},\n");
    let processed = processed.replace("],", "],\n");
    let processed = processed.replace('{', "{\n");
    let mut show_it = false;

    let mut acc = vec!["{".to_string()];
    for line in processed.lines() {
        if line.contains("carsPaginated") {
            show_it = true;
        } else if line.contains("pagination") {
            break;
        }

        if show_it {
            if line.trim().is_empty() {
                continue;
            }
            acc.push(line.to_string());
            continue;
        }
    }
    let len = acc.len();
    let last = acc[len - 1].clone();
    let last = last.replace("}],", "}]");
    acc[len - 1] = last.to_string();
    acc.push("}\n}".to_string());
    let lines = acc.join("\n");
    let json = match serde_json::from_str::<PaginatedCars>(&lines) {
        Ok(json) => json,
        Err(e) => {
            error!("Error: {:?}", e);
            let l = e.line();
            error!("Line: {:?}", lines.lines().nth(l - 1).unwrap());
            return vec![];
        }
    };
    json.paginated.cars
}

pub fn get_scripts(html: &str) -> Vec<String> {
    let document = Html::parse_document(html);

    let script_selector = Selector::parse("script").unwrap();
    let scripts = document
        .select(&script_selector)
        .map(|script| script.inner_html())
        .collect::<Vec<String>>()
        .into_iter()
        .filter(|s| s.contains("announcedAsNew"))
        .collect();
    scripts
}

pub fn get_vehicles(content: &str) -> Vec<AutoUncleVehicle> {
    let mut vehicles = parse_vehicles(content);
    for v in &mut vehicles {
        v.equipment = v.featuredAttributesEquipment.clone();
        v.equipment.extend(v.featuredAttributesNonEquipment.clone());
    }
    vehicles
}

fn extract_json(js_content: &str) -> Vec<AutoUncleVehicle> {
    let re = Regex::new(r"\{[\s\S]*}}}}").unwrap(); // Adjust regex to capture JSON correctly
    let jsons: Vec<&str> = re
        .find_iter(js_content)
        .map(|mat| mat.as_str().trim_end_matches(");")) // Removing trailing characters if any
        .collect();
    for js in jsons {
        let js = js.to_string().replace("\\\"", "\""); // Correcting escape sequences for quotes
        let js = js.replace(r"\\", r"\");
        let js = js + "}";
        match serde_json::from_str::<Root>(&js) {
            Ok(json) => return json.dynamicScriptData.cars_search.carsPaginated.cars,
            Err(e) => {
                error!("Failed to deserialize: {:?}", e);
                continue;
                // This will show the error why deserialization failed
            }
        }
    }
    vec![]
}

pub fn parse_vehicles(content: &str) -> Vec<AutoUncleVehicle> {
    let script_selector = Selector::parse("script").unwrap();
    let mut vehicles = vec![];
    let html = Html::parse_document(content);

    let scripts = html
        .select(&script_selector)
        .map(|script| script.inner_html())
        .collect::<Vec<String>>()
        .into_iter()
        .filter(|s| s.contains("announcedAsNew"))
        .collect::<Vec<String>>();

    for s in scripts {
        let found = extract_json(&s);
        if found.is_empty() {
            continue;
        }
        vehicles.extend(found);
    }
    vehicles
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
                re.find(&js).and_then(|caps| -> Option<()> {
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

    use crate::{utils::helpers::configure_log4rs, LOG_CONFIG};

    use super::*;

    #[test]
    fn test_list_vehicles_from_text() {
        configure_log4rs(&LOG_CONFIG);
        let content = fs::read_to_string("resources/test-data/autouncle/nl_1.html").unwrap();
        let vehicles = get_vehicles(&content);
        info!("Vehicles: {}", vehicles.len());
        for v in &vehicles {
            info!("Vehicle: {:?}", v);
        }
    }

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
}
