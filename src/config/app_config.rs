use std::{fs::File, io::Read};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct AppConfig {
    #[serde(rename = "scraper-config")]
    scraper_config: String,
    #[serde(rename = "downloaded-data-dir")]
    data_dir: String,
    #[serde(rename = "log4rs-config-dir")]
    log4rs_config: String,
    #[serde(rename = "num-threads")]
    num_threads: usize,
}

impl AppConfig {
    pub fn from_file(file_name: &str) -> Self {
        let mut file = File::open(file_name).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let config: AppConfig = serde_yaml::from_str(&contents).unwrap();
        config
    }

    pub fn get_scraper_config(&self) -> &str {
        &self.scraper_config
    }

    pub fn get_data_dir(&self) -> &str {
        &self.data_dir
    }

    pub fn get_log4rs_config(&self) -> &str {
        &self.log4rs_config
    }

    pub fn get_num_threads(&self) -> usize {
        self.num_threads
    }
}

#[cfg(test)]
mod tests {

    use crate::config::app_config::AppConfig;

    #[test]
    fn test_url() {
        let config = AppConfig::from_file("config/config.yml");
        println!("{:#?}", config);
        assert_eq!(
            "config/mobile/scraper-config.yml",
            config.get_scraper_config()
        );
        assert_eq!("resources/data", config.get_data_dir());
        assert_eq!("config/loggers", config.get_log4rs_config());
        assert_eq!(8, config.get_num_threads());
    }
}
