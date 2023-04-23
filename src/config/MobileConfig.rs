use std::fs;

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Link {
    pub name: String,
    pub link: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Links {
    pub name: String,
    pub links: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Dealer {
    pub all: Link,
    pub sold: Link,
    pub new: Links,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    dealers: Dealer,
    private: Dealer,
}

impl Config {
    pub fn from_file(file_name: &str) -> Self {
        if let Ok(contents) = fs::read_to_string(file_name) {
            if let Ok(config) = serde_yaml::from_str(&contents) {
                return config;
            }
        }
        Config {
            ..Default::default()
        }
    }

    pub fn write_config_file(&self, file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let contents = serde_yaml::to_string(&self)?;
        fs::write(file_name, contents)?;
        Ok(())
    }

    pub fn dealears(&self) -> Dealer {
        self.dealers.clone()
    }

    pub fn private(&self) -> Dealer {
        self.private.clone()
    }
}
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct LinkData {
    pub name: String,
    pub link: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct LinksData {
    pub name: String,
    pub links: Vec<String>,
}
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ConfigData {
    pub dealear_type: String,
    pub all: LinkData,
    pub sold: LinkData,
    pub new: LinkData,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Mobile {
    pub config: Vec<ConfigData>,
}

impl Mobile {
    pub fn from_file(file_name: &str) -> Self {
        if let Ok(contents) = fs::read_to_string(file_name) {
            let yml_value: Vec<Value> = serde_yaml::from_str(&contents).unwrap();
            let config: Vec<ConfigData> = yml_value
                .iter()
                .map(|v| v["config"].clone())
                .map(|config_value| serde_yaml::from_value::<ConfigData>(config_value).unwrap())
                .collect();
            return Mobile { config };
        }
        Mobile {
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        config::MobileConfig::{Config, Mobile},
        LISTING_URL,
    };

    #[test]
    fn test_read_config_file() {
        let file_name = "config/config.yml";
        let config = Config::from_file(file_name);
        assert_eq!(config.dealers.all.link, "s4mb6z");
        assert_eq!(config.dealers.all.name, "ALL");

        assert_eq!(config.private.all.link, "s4mbhc");
        assert_eq!(config.private.all.name, "ALL");
        assert_eq!(config.private.new.links[0], "s4mc5e");
    }

    #[test]
    fn test_mobile_config_file() {
        let file_name = "config/mobile_config.yml";
        let mobile: Mobile = Mobile::from_file(file_name);
        println!("{:?}", mobile);
        assert_eq!(mobile.config[0].all.link, "s4mb6z");
    }

    #[test]
    fn test_url() {
        let file_name = "config/mobile_config.yml";
        let mobile: Mobile = Mobile::from_file(file_name);
        let listing_url = format!(
            "{}{}",
            LISTING_URL,
            format!("&slink={}&f1={}", mobile.config[0].all.link, "1")
        );
        println!("{}", listing_url);
    }
}
