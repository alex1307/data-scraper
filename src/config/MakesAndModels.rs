use std::{
    collections::HashMap,
    fs,
    sync::{Once, RwLock},
};

use lazy_static::lazy_static;
use log::info;

use crate::{utils::helpers::configure_log4rs, LOG_CONFIG};

lazy_static! {
    static ref INIT_MAKE_AND_MODESL: Once = Once::new();
    pub static ref MAKE_MODEL_SERIES: RwLock<HashMap<String, HashMap<String, Vec<String>>>> =
        RwLock::new(HashMap::new());
    pub static ref MAKE_MODEL: RwLock<HashMap<String, Vec<String>>> = RwLock::new(HashMap::new());
}

pub fn load_makes_and_models() {
    configure_log4rs(&LOG_CONFIG);
    INIT_MAKE_AND_MODESL.call_once(|| {
        let json_data = fs::read_to_string("config/make_and_models.json").unwrap();
        let parsed: Result<HashMap<String, HashMap<String, Vec<String>>>, serde_json::Error> =
            serde_json::from_str(&json_data);
        match parsed {
            Ok(data) => {
                info!("Successfully loaded makes, series and models");
                let mut make_model_checker = MAKE_MODEL_SERIES.write().unwrap();
                *make_model_checker = data.clone();
                let mut make_model = MAKE_MODEL.write().unwrap();
                for make in data.keys() {
                    let make_models = get_models(make);
                    make_model.insert(make.to_string(), make_models);
                }
            }
            Err(e) => {
                panic!("Failed to parse make and models: {}", e);
            }
        }
    });
}

pub fn get_models(make: &str) -> Vec<String> {
    let make_model_checker = MAKE_MODEL_SERIES.read().unwrap();
    match make_model_checker.get(make) {
        Some(models) => models
            .values()
            .flatten()
            .map(|s| s.trim().to_lowercase())
            .collect(),
        None => {
            info!("No models found for make {}", make);
            vec![]
        }
    }
}
