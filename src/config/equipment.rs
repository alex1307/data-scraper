use lazy_static::lazy_static;
use serde_yaml::Value;
use std::collections::HashMap;

lazy_static! {
    pub static ref EQUIPMENT: HashMap<u64, Vec<String>> = {
        let yaml_str = std::fs::read_to_string("config/equipment-list.yml").unwrap();
        let yaml_value: Value = serde_yaml::from_str(&yaml_str).unwrap();
        match yaml_value["equipment"].clone() {
            Value::Mapping(map) => {
                let mut result_map: HashMap<u64, Vec<String>> = HashMap::new();
                for (key, value) in map {
                    let key = key.as_i64().unwrap() as u64;
                    let value = value.as_sequence().unwrap();
                    let mut values = Vec::new();
                    for v in value {
                        match v.as_str() {
                            Some(v) => values.push(v.trim().to_lowercase()),
                            None => continue,
                        }
                    }
                    result_map.insert(key, values);
                }
                result_map
            }

            _ => HashMap::new(),
        }
    };
    pub static ref EQUIPMENT_REVERSED: HashMap<String, u64> = {
        let yaml_str = std::fs::read_to_string("config/equipment-list.yml").unwrap();
        let yaml_value: Value = serde_yaml::from_str(&yaml_str).unwrap();
        match yaml_value["equipment"].clone() {
            Value::Mapping(map) => {
                let mut result_map: HashMap<String, u64> = HashMap::new();
                for (key, value) in map {
                    let key = key.as_i64().unwrap() as u64;
                    let value = value.as_sequence().unwrap();
                    for v in value {
                        match v.as_str() {
                            Some(v) => result_map.insert(v.trim().to_lowercase(), key),
                            None => continue,
                        };
                    }
                }
                result_map
            }
            _ => HashMap::new(),
        }
    };
}

pub fn get_equipment_as_u64(values: Vec<String>) -> u64 {
    let mut sum = 0;
    for value in values {
        if let Some(key) = EQUIPMENT_REVERSED.get(&value.trim().to_lowercase()) {
            sum += 2_u64.pow(*key as u32);
        }
    }
    sum
}

pub fn get_values_by_equipment_id(equipment_id: u64) -> Vec<String> {
    let mut equipment = Vec::new();
    for (key, values) in EQUIPMENT.iter() {
        let mask = 2_u64.pow(*key as u32);
        if equipment_id & mask == mask {
            equipment.push(values.first().unwrap().to_string());
        }
    }
    equipment
}

#[cfg(test)]
mod test {
    use std::vec;

    use log::info;

    use crate::{
        config::Equipment::{get_equipment_as_u64, get_values_by_equipment_id},
        utils::helpers::configure_log4rs,
        LOG_CONFIG,
    };

    use super::EQUIPMENT;

    #[test]
    fn get_4wd_pilot_test() {
        let values = vec![
            "4x4".to_string(),
            "Система за контрол на скоростта (автопилот)".to_string(),
        ];

        let equipment_id = get_equipment_as_u64(values);
        println!("{}", equipment_id); // Output: 7
        assert_eq!(equipment_id, 20971520);
        let values = get_values_by_equipment_id(equipment_id);
        assert_eq!(values.len(), 2,);
    }

    #[test]
    fn test_get_all_equipments() {
        configure_log4rs(&LOG_CONFIG);
        let equipment = &EQUIPMENT;
        for (key, value) in equipment.iter() {
            info!("{}: {:?}", key, value);
        }
        let reverse = &super::EQUIPMENT_REVERSED;
        for (key, value) in reverse.iter() {
            info!("{}: {:?}", key, value);
        }
    }
}
