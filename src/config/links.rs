use std::{fs::File, io::Read};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Link {
    pub name: String,
    pub link: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct LinkData {
    pub name: String,
    pub link: String,
    pub scrape: bool,
    pub filter: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct LinksData {
    pub name: String,
    pub links: Vec<String>,
}
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ConfigData {
    pub dealear_type: String,
    pub file_name: String,
    pub links: Vec<LinkData>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct DealerConfig {}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Mobile {
    pub config: Vec<ConfigData>,
}

impl Mobile {
    pub fn from_file(file_name: &str) -> Self {
        let mut file = File::open(file_name).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let config: Mobile = serde_yaml::from_str(&contents).unwrap();
        config
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read};

    use crate::config::links::Mobile;

    #[test]
    fn test_url() {
        let mut file = File::open("config/mobile_config.yml").unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let config: Mobile = serde_yaml::from_str(&contents).unwrap();
        // for v in config.iter() {
        //     println!("{:#?}", v);
        //     let config_data: ConfigData = serde_yaml::from_value(v["config"].clone()).unwrap();
        //     println!("{:#?}", config_data);
        // }
        println!("{:#?}", config);
    }

    #[test]
    fn mobile_test() {
        let config = Mobile::from_file("config/mobile_config.yml");
        println!("{:#?}", config);
    }
}
