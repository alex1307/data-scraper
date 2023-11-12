use lazy_static::lazy_static;
use serde_yaml::Value;
use std::collections::HashMap;

lazy_static! {
    pub static ref MOBILE_BG_EQUIPMENT: HashMap<u64, String> = {
        let yaml_str = std::fs::read_to_string("config/equipment.yml").unwrap();
        let yaml_value: Value = serde_yaml::from_str(&yaml_str).unwrap();
        match yaml_value["equipment"].clone() {
            Value::Mapping(map) => {
                let mut result_map: HashMap<u64, String> = HashMap::new();
                for (key, value) in map {
                    let key = key.as_i64().unwrap() as u64;
                    let value = value.as_str().unwrap().to_owned();
                    result_map.insert(key, value);
                }
                result_map
            }

            _ => HashMap::new(),
        }
    };
    pub static ref CARS_BG_EQUIPMENT: HashMap<u64, String> = {
        let yaml_str = std::fs::read_to_string("config/cars-bg-equipment.yml").unwrap();
        let yaml_value: Value = serde_yaml::from_str(&yaml_str).unwrap();
        match yaml_value["equipment"].clone() {
            Value::Mapping(map) => {
                let mut result_map: HashMap<u64, String> = HashMap::new();
                for (key, value) in map {
                    let key = key.as_i64().unwrap() as u64;
                    let value = value.as_str().unwrap().to_owned();
                    result_map.insert(key, value);
                }
                result_map
            }

            _ => HashMap::new(),
        }
    };
}

pub fn get_keys_by_values(values: &[&str]) -> Vec<u64> {
    let mut keys = Vec::new();
    for (key, value) in MOBILE_BG_EQUIPMENT.iter() {
        if values.contains(&value.as_str()) {
            keys.push(*key);
        }
    }
    keys
}

pub fn get_equipment_as_u64(values: Vec<String>, equipment_map: &HashMap<u64, String>) -> u64 {
    let mut sum = 0;
    for (key, value) in equipment_map.iter() {
        if values.contains(value) {
            sum += 2_u64.pow(*key as u32);
        }
    }
    sum
}

pub fn get_values_by_equipment_id(equipment_id: u64, map: &HashMap<u64, String>) -> Vec<String> {
    let mut values = Vec::new();
    for (key, value) in map.iter() {
        let mask = 2_u64.pow(*key as u32);
        if equipment_id & mask == mask {
            values.push(value.to_string());
        }
    }
    values
}

#[cfg(test)]
mod test {
    use std::vec;

    use crate::config::equipment::{
        get_equipment_as_u64, get_values_by_equipment_id, MOBILE_BG_EQUIPMENT,
    };

    #[test]
    fn get_petrol_automatic() {
        let values = vec![
            "Бензинов".to_string(),
            "Ръчна".to_string(),
            "4x4".to_string(),
            "Система за контрол на скоростта (автопилот)".to_string(),
        ];

        let equipment_id = get_equipment_as_u64(values, &MOBILE_BG_EQUIPMENT);
        println!("{}", equipment_id); // Output: 7
        assert_eq!(equipment_id, 20971586);
        let values = get_values_by_equipment_id(equipment_id, &MOBILE_BG_EQUIPMENT);
        assert_eq!(
            values,
            vec![
                "Бензинов",
                "Ръчна",
                "4x4",
                "Система за контрол на скоростта (автопилот)"
            ]
        );
    }
}
