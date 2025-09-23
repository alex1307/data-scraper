use std::{collections::HashMap, fmt::Write, vec};

use log::{error, info};
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
            info!("json: {:?}", json);
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
mod tests {
    use super::*;
    use crate::LOG_CONFIG;
    use crate::utils::helpers::configure_log4rs;

    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_find_json_bounds() {
        configure_log4rs(&LOG_CONFIG);
        //read content from file resources/test_data/autouncle.ro.json
        let path = PathBuf::from("resources/test-data/autouncle/autouncle.ro.json");
        let content = fs::read_to_string(path).expect("Failed to read file");
        let result = process_js(content);
        assert!(result.is_some());
        let car_data = result.unwrap();
        info!("Car data: {:?}", car_data);
    }
}
