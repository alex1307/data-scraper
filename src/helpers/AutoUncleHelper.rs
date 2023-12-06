use scraper::{Html, Selector};

fn get_scripts(html: &str) -> Vec<String> {
    let document = Html::parse_document(&html);
    let script_selector = Selector::parse("script").unwrap();
    let scripts = document
        .select(&script_selector)
        .map(|script| script.inner_html())
        .collect::<Vec<String>>();
    scripts
}

#[cfg(test)]
mod auto_uncle_tests {
    use std::{
        collections::HashMap,
        fs,
        io::{self, Read},
        path::Path,
        vec,
    };

    use log::{error, info};
    use log4rs::filter;
    use regex::Regex;
    use serde::{Deserialize, Serialize};
    use serde_yaml::Value;

    use crate::{utils::helpers::configure_log4rs, LOG_CONFIG};

    use super::*;

    #[derive(Serialize, Deserialize, Debug)]
    struct Vehicles{
        carsPaginated: Cars,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Cars{
        cars: Vec<Car>,
    }

    
    #[derive(Serialize, Deserialize, Debug)]
    struct Car {
        announcedAsNew: bool,
        auRating: Option<u8>,
        brand: Option<String>,
        body: Option<String>,
        displayableFuelConsumption: Option<String>,
        carModel: Option<String>,
        co2Emission: Option<f64>,
        createdAt: Option<String>,
        currency: Option<String>,
        doors: Option<u8>,
        electricDriveRange: Option<f64>,
        engineSize: Option<f64>,
        equipmentVariant: Option<String>,
        estimatedPrice: Option<u32>,
        featuredAttributesEquipment: Vec<String>,
        featuredAttributesNonEquipment: Vec<String>,
        fuel: Option<String>,
        fuelEconomy: Option<f64>,
        hasAutoGear: Option<bool>,
        headline: Option<String>,
        hp: Option<u16>,
        id: String,
        isFeatured: Option<bool>,
        km: u32,
        kw: u16,
        localizedFuelEconomy: Option<f64>,
        localizedFuelEconomyLabel: String,
        location: String,
        modelGeneration: String,
        noRatingReasons: Vec<String>,
        outgoingPath: String,
        price: u32,
        regMonth: Option<u8>,
        sourceName: String,
        updatedAt: String,
        vdpPath: String,
        year: u16,
        youSaveDifference: u32,
        laytime: u8,
        sellerKind: String,
        isElectric: bool,
        priceChange: i32,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Brand {
        name: String,
        // Add other fields from the Brand object
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Offer {
        price: f64,
        priceCurrency: String,
        // Add other fields from the Offer object
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Product {
        name: String,
        brand: Brand,
        offers: Offer,
        // Add other fields from the Product object
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct ItemList {
        numberOfItems: i32,
        itemListElement: Vec<Product>,
        // Add other fields from the ItemList object
    }

    fn read_and_parse_json<P: AsRef<Path>>(path: P) -> Result<Value, io::Error> {
        let mut file = fs::File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let data = serde_json::from_str::<Value>(&contents)?;
        Ok(data)
    }

    #[test]
    fn test_get_scripts() {
        configure_log4rs(&LOG_CONFIG);
        let content = fs::read_to_string("resources/test-data/autouncle/3.html").unwrap();
        let scripts = get_scripts(&content);
        info!("Filtered: {:?}", scripts.len());
        for s in scripts {
            
            if s.contains("self.__next_f.push") && s.contains("carsPaginated") {
                let s = s.replace(r#"\""#, r#"""#);
                let s = s.replace("\\", "");
                let s = s.replace(r#"\\"#, r#"\"#);
                let s = s.replace(r#"\n"#, r#""#);
                let s = s.replace(r#"\t"#, r#""#);
                let s = s.replace(r#"\r"#, r#""#);
                let s = s.replace(r#""""#, r#"""#);
                let s = s.replace("}],", "}],\n");
                let s = s.replace("},", "},\n");
                let s = s.replace("],", "],\n");
                info!("Found: {:?}", s.len());
                let mut show_it = false;
                let mut counter = 0;
                let mut acc = vec!["{".to_string()];
                for line in s.lines() {
                    if line.contains("carsPaginated") {
                        show_it = true;
                    } else if line.contains("pagination") {
                        show_it = false;
                        break;
                    }

                    if show_it {
                        counter += 1;
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
                info!("Found lines: {:?}", counter);
                let lines = acc.join("\n");
                info!("-------------------");
                info!("{}", lines);
                let json = serde_json::from_str::<Vehicles>(&lines).unwrap();
                info!("{:?}", json);
                info!("-------------------");
            }
        }
    }
}
