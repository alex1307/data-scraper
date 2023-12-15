#[cfg(test)]
mod mobile_de_tests {
    use std::{fs, str::FromStr};

    use log::info;

    use crate::{model::{MobileDe::MobileDeResults, enums::{Engine, Gearbox}}, utils::helpers::configure_log4rs, LOG_CONFIG};

    #[test]
    fn parse_json() {
        configure_log4rs(&LOG_CONFIG);
        let content =
            fs::read_to_string("resources/test-data/mobile.de/mobile_listing.json").unwrap();
        let json = serde_json::from_str::<MobileDeResults>(&content).unwrap();
        info!("{:?}", json);
    }

    #[test]
    fn get_json_from_html() {
        configure_log4rs(&LOG_CONFIG);
        let mut content =
            fs::read_to_string("resources/test-data/mobile.de/my_file_3.html").unwrap();
        content = content.replace('\u{2009}', " ");
        content = content.replace('\u{a0}', "");
        info!("{:?}", content.len());
        if let Some(start_idx) = content.find("window.__INITIAL_STATE__ = ") {
            let start_idx = start_idx + "window.__INITIAL_STATE__ = ".len();
            if let Some(end_idx) = content.find("window.__PUBLIC_CONFIG__") {
                let json = &content[start_idx..end_idx];
                let json = serde_json::from_str::<MobileDeResults>(json).unwrap();
                info!("{:?}", json);
            }
        }
    }

    #[test]
    fn get_thresholds() {
        configure_log4rs(&LOG_CONFIG);
        let source = vec![
            "13.000\u{a0}€",
            "15.600\u{a0}€",
            "16.500\u{a0}€",
            "18.400\u{a0}€",
            "21.100\u{a0}€",
            "22.000\u{a0}€",
        ];
        let mut thresholds = vec![];
        for s in &source {
            let number = s.chars().filter(|c| c.is_digit(10)).collect::<String>();
            info!("{:?}", number);
            thresholds.push(number.parse::<u32>().unwrap());
        }
        assert_eq!(thresholds.len(), source.len());
    }

    #[test]
    fn parse_attributes() {
        configure_log4rs(&LOG_CONFIG); 
        let attr1 = vec![
            "FR 04/2023 • 8,000km • 215kW(292Hp)",
            "Demonstration Vehicle • SUV / Off-road Vehicle / Pickup Truck • Availability: From Apr 30, 2024 • Electric • Automatic • HU 04/2026 • 4/5 Doors",
            "18.3 kWh/100km (comb.)* • 0g CO₂/km (comb.)*"];

        let attr2 = vec![
            "EZ 06/2018 • 105.800km • 110kW(150PS)",
            "Kombi • Diesel • Schaltgetriebe • HU Neu • 4/5 Türen",
            "ca. 5,1 l/100km (komb.) • ca. 133 g CO₂/km (komb.)",
        ];

        let attributes = attr2.clone();
        let flattened_attributes: Vec<String> = attributes
            .iter()
            .flat_map(|a| a.split(" • "))
            .map(|s| s.to_string())
            .collect();
        
        info!("{:?}", flattened_attributes);
        let prod_year = flattened_attributes[0]
            .split(" ")
            .collect::<Vec<&str>>()[1]
            .split("/")
            .collect::<Vec<&str>>();
        info!("{:?}", prod_year);
        if attributes == attr1 {
            assert_eq!(prod_year[0], "04");
            assert_eq!(prod_year[1], "2023");
        } else {
            assert_eq!(prod_year[0], "06");
            assert_eq!(prod_year[1], "2018");
        }
        

        let milage = flattened_attributes[1]
            .chars()
            .filter(|c| c.is_digit(10))
            .collect::<String>()
            .parse::<u32>()
            .unwrap();
        info!("{:?}", milage);

        if attributes == attr1 {
            assert_eq!(milage, 8000);
        } else {
            assert_eq!(milage, 105800);
        }

        
        if flattened_attributes[2].contains("KW") {
            let kw_ps = flattened_attributes[2]
                .split("KW")
                .collect::<Vec<&str>>();
            let mut power = vec![];
            for s in kw_ps {
                let number = s
                    .chars()
                    .filter(|c| c.is_digit(10))
                    .collect::<String>()
                    .parse::<u32>()
                    .unwrap();
                power.push(number);
            }
            if attributes == attr1 {
                assert_eq!(power[0], 215);
                assert_eq!(power[1], 292);
            } else {
                assert_eq!(power[0], 110);
                assert_eq!(power[1], 150);
            }
        }
        let mut engine: Engine = Engine::NotAvailable;
        let mut gearbox: Gearbox = Gearbox::NotAvailable;
        for attr in &flattened_attributes{
            engine = Engine::from_str(attr.as_str()).unwrap();
            if Engine::NotAvailable == engine {
                continue;
            }else {
                break;
            }
        }


        for attr in &flattened_attributes{
            gearbox = Gearbox::from_str(attr.as_str()).unwrap();
            if Gearbox::NotAvailable == gearbox {
                continue;
            }else {
                break;
            }
        }

        if attributes == attr1 {
            assert_eq!(engine, Engine::Electric);
        }else {
            assert_eq!(engine, Engine::Diesel);
        }

        if attributes == attr1 {
            assert_eq!(gearbox, Gearbox::Automatic);
        }else {
            assert_eq!(gearbox, Gearbox::Manual);
        }
        

        
    }
}
