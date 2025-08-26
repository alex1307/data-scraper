#[cfg(test)]
mod tests {
    use crate::{
        LOG_CONFIG,
        utils::{
            MakeAndModelsUtils::{
                MakeData, Model, build_make_model_map, find_unmatched_models,
                save_make_data_to_csv, tokenize, update_make_data,
            },
            helpers::configure_log4rs,
        },
    };

    use std::{
        collections::{HashMap, HashSet},
        fs::File,
        io::Write,
    };

    use csv::ReaderBuilder;
    use headless_chrome::Browser;
    use log::info;
    use reqwest::get;
    use serde::Deserialize;

    #[test]
    fn test_make_and_models() {
        configure_log4rs(&LOG_CONFIG);
        let file = File::open("data/vehicles-2025-04-11.csv").unwrap();
        let mut rdr = ReaderBuilder::new().delimiter(b';').from_reader(file);

        let headers = rdr.headers().unwrap().clone();
        let make_idx = headers.iter().position(|h| h == "make").unwrap();
        let model_idx = headers.iter().position(|h| h == "model").unwrap();
        let source_idx = headers.iter().position(|h| h == "source").unwrap();
        let mut unique_models = HashSet::new();
        for result in rdr.records() {
            let record = result.unwrap();
            let source = record.get(source_idx).unwrap_or("").trim();
            if source == "mobile.bg" || source == "mobile.de" {
                continue;
            }
            let make = record.get(make_idx).unwrap_or("").trim().to_string();
            let model = record.get(model_idx).unwrap_or("").trim().to_string();
            let tags: Vec<String> = tokenize(&model);

            let make_entry = MakeData {
                id: format!(
                    "{}{}",
                    make.to_uppercase().trim(),
                    model.to_uppercase().trim()
                ),
                make: make.clone(),
                model: model.clone(),
                tags: tags.clone(),
            };
            unique_models.insert(make_entry.clone());
        }

        info!("Unique models: {:?}", &unique_models.len());
        let mut make_model_map: HashMap<String, Vec<Model>> = HashMap::new();
        for make_entry in unique_models {
            let make = make_entry.make;
            let model = make_entry.model;
            let tags = make_entry.tags;

            let model_entry = Model {
                value: model.clone(),
                tags: tags.clone(),
            };

            make_model_map
                .entry(make)
                .or_insert_with(Vec::new)
                .push(model_entry);
        }

        // // Convert HashSet to Vec and sort for consistency

        let json = serde_json::to_string_pretty(&make_model_map).unwrap();

        // // Write to file
        let mut file = File::create("model_lookup.json").unwrap();
        save_make_data_to_csv(&make_model_map, "model_lookup.csv").unwrap();

        file.write_all(json.as_bytes()).unwrap();
        info!("Makes: {}", make_model_map.keys().len());
        println!("✅ Model lookup table saved as 'model_lookup.json'");
    }

    #[test]
    fn test_make_and_models_mobile_de() {
        let make_model_map =
            build_make_model_map(vec!["data/vehicles-2025-04-11.csv"], vec!["mobile.de"]);
        update_make_data(&make_model_map, "model_lookup_mobile_de.json");
    }

    #[test]
    fn test_find_unmatched_models_fixed() {
        configure_log4rs(&LOG_CONFIG);
        let mobile_de_map =
            build_make_model_map(vec!["data/vehicles-2025-04-11.csv"], vec!["mobile.de"]);
        let autouncle_map = build_make_model_map(
            vec!["data/vehicles-2025-04-11.csv"],
            vec![
                "autouncle.ro",
                "autouncle.nl",
                "autouncle.fr",
                "autouncle.it",
                "autouncle.ch",
            ],
        );

        info!("Mobile.de makes: {}", mobile_de_map.keys().len());
        info!("Autouncle makes: {}", autouncle_map.keys().len());

        let unmatched_models = find_unmatched_models(&mobile_de_map, &autouncle_map);
        info!("Unmatched models: {}", unmatched_models.len());

        let mut missing_models = Vec::new();
        let mut found_models = 0;
        for (make, models) in unmatched_models.iter() {
            if let Some(autouncle_models) = autouncle_map.get(make) {
                for model in models {
                    let mut found = false;
                    for autouncle_model in autouncle_models {
                        if model.tags.first() == autouncle_model.tags.first()
                            && model.tags.len() == autouncle_model.tags.len()
                        {
                            found_models += 1;
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        missing_models.push((make.clone(), model.value.clone()));
                    }
                }
            } else {
                // Entire make missing
                for model in models {
                    missing_models.push((make.clone(), model.value.clone()));
                }
            }
        }
        println!("✅ Found models count: {}", found_models);
        println!("❌ Missing models count: {}", missing_models.len());
        for (make, model) in &missing_models {
            println!("Missing -> Make: {}, Model: {}", make, model);
        }

        assert!(
            missing_models.len() <= 50,
            "Too many missing models: {}",
            missing_models.len()
        );
    }

    #[test]
    fn test_find_unmatched_modesl() {
        configure_log4rs(&LOG_CONFIG);
        let mobilde_de_map =
            build_make_model_map(vec!["data/vehicles-2025-04-11.csv"], vec!["mobile.de"]);
        let autouncle_map = build_make_model_map(
            vec!["data/vehicles-2025-04-11.csv"],
            vec![
                "autouncle.ro",
                "autouncle.nl",
                "autouncle.fr",
                "autouncle.it",
                "autouncle.ch",
            ],
        );
        info!("Mobile.de makes: {}", mobilde_de_map.keys().len());
        info!("Autouncle makes: {}", autouncle_map.keys().len());

        let unmatched_models = find_unmatched_models(&mobilde_de_map, &autouncle_map);
        info!("Unmatched models: {}", unmatched_models.len());
        //count exact and similar matches
        let mut exact_matches = 0;
        let mut model_matches = 0;
        let mut similar_matches = 0;
        let mut not_found = 0;
        for (make, models) in unmatched_models.iter() {
            // info!("Make: {}", make);
            for model in models {
                // info!("  Model: {:?}", model);
                //check if model's first tag is found in autouncle_map
                if let Some(autouncle_models) = autouncle_map.get(make) {
                    for autouncle_model in autouncle_models {
                        //check if the first tag of the model is in autouncle_model's tags
                        if model.tags[0] == autouncle_model.tags[0] {
                            if model.tags.len() == autouncle_model.tags.len() {
                                // info!(
                                //     "Exact match-> Found match: {} == {}",
                                //     autouncle_model.value, model.value
                                // );
                                exact_matches += 1;
                            } else {
                                // info!(
                                //     "Exact model match-> Found match: {} == {}",
                                //     autouncle_model.value, model.value
                                // );
                                model_matches += 1;
                            }

                            // info!(
                            //     "Exact match-> Found match: {} == {}",
                            //     autouncle_model.value, model.value
                            // );
                        } else if autouncle_model.tags.contains(&model.tags[0]) {
                            similar_matches += 1;
                            // info!("Similar match-> Found match: {}", autouncle_model.value);
                        } else {
                            not_found += 1;
                            // info!("Not found: {}", model.value);
                        }
                    }
                }
            }
        }

        info!("Exact matches: {}", exact_matches);
        info!("Model matches: {}", model_matches);
        info!("Similar matches: {}", similar_matches);
        info!("Not found: {}", not_found);
    }

    #[test]
    fn test_headless_chrome() {
        #[derive(Deserialize)]
        struct DebuggerInfo {
            webSocketDebuggerUrl: String,
        }

        configure_log4rs(&LOG_CONFIG);

        // Step 1: Get the actual WebSocket URL from the running Chrome
        let resp = tokio::runtime::Runtime::new().unwrap().block_on(async {
            get("http://localhost:9223/json/version")
                .await
                .expect("Failed to fetch Chrome DevTools version")
                .json::<DebuggerInfo>()
                .await
                .expect("Failed to parse debugger info")
        });
        info!("WebSocket URL: {}", resp.webSocketDebuggerUrl);

        let browser =
            Browser::connect(resp.webSocketDebuggerUrl).expect("Failed to connect to browser");
        let tab = browser.new_tab().expect("Failed to open new tab");

        tab.navigate_to("https://www.autouncle.ro/en/cars_search?s%5Bmax_km%5D=200000&s%5Bmax_year%5D=2024&s%5Bmin_price%5D=90000&s%5Bmin_year%5D=2024&s%5Bnot_damaged%5D=true&s%5Bseller_kind%5D=Dealer&s%5Bwith_ratings%5D%5B%5D=5")
            .expect("Navigation failed");
        tab.wait_until_navigated().expect("Page did not load");

        let html = tab.get_content().expect("Failed to get content");
        println!("{}", html);
    }

    #[test]
    fn test_generate_csv_with_make_model_id_from_csv_lookup() {
        use csv::{ReaderBuilder, WriterBuilder};
        use std::collections::HashMap;
        use std::fs::File;

        configure_log4rs(&LOG_CONFIG);

        // Step 1: Load lookup from model_lookup.csv
        let lookup_file = File::open("model_lookup.csv").expect("Lookup CSV not found");
        let mut lookup_rdr = ReaderBuilder::new()
            .delimiter(b';')
            .from_reader(lookup_file);
        let lookup_headers = lookup_rdr.headers().unwrap().clone();

        let lookup_make_idx = lookup_headers
            .iter()
            .position(|h| h == "make")
            .expect("Missing 'make' column in lookup CSV");
        let lookup_model_idx = lookup_headers
            .iter()
            .position(|h| h == "model")
            .expect("Missing 'model' column in lookup CSV");

        let mut lookup_map: HashMap<(String, String), String> = HashMap::new();

        for result in lookup_rdr.records() {
            let record = result.expect("Failed to read lookup record");
            let make = record.get(lookup_make_idx).unwrap_or("").trim().to_string();
            let model = record
                .get(lookup_model_idx)
                .unwrap_or("")
                .trim()
                .to_string();
            let id = format!("{}{}", make.to_uppercase(), model.to_uppercase());
            lookup_map.insert((make.clone(), model.clone()), id);
        }

        // Step 2: Load input vehicles CSV
        let file_in = File::open("data/vehicles-2025-04-11.csv").expect("Input CSV not found");
        let mut rdr = ReaderBuilder::new().delimiter(b';').from_reader(file_in);
        let headers = rdr.headers().unwrap().clone();

        let make_idx = headers
            .iter()
            .position(|h| h == "make")
            .expect("Missing 'make' column in vehicles CSV");
        let model_idx = headers
            .iter()
            .position(|h| h == "model")
            .expect("Missing 'model' column in vehicles CSV");

        // Step 3: Prepare output CSV writer
        let output_path = "data/vehicles-2025-04-11_with_ids_test.csv";
        let file_out = File::create(output_path).expect("Failed to create output CSV");
        let mut wtr = WriterBuilder::new().delimiter(b';').from_writer(file_out);

        // Step 4: Write new headers
        let mut new_headers = headers.clone();
        new_headers.push_field("make_model_id");
        wtr.write_record(&new_headers)
            .expect("Failed to write headers");

        // Step 5: Process each input record
        for result in rdr.records() {
            let record = result.expect("Failed to read vehicle record");
            let mut new_record: Vec<String> = record.iter().map(|s| s.to_string()).collect();

            let make = record.get(make_idx).unwrap_or("").trim().to_string();
            let model = record.get(model_idx).unwrap_or("").trim().to_string();

            let make_model_id = lookup_map
                .get(&(make.clone(), model.clone()))
                .cloned()
                .unwrap_or_else(|| "".to_string());

            new_record.push(make_model_id);

            wtr.write_record(&new_record)
                .expect("Failed to write output record");
        }

        wtr.flush().expect("Flush failed");

        // Step 6: Validate output
        let file_out = File::open(output_path).expect("Failed to open output for validation");
        let mut rdr_out = ReaderBuilder::new().delimiter(b';').from_reader(file_out);

        let out_headers = rdr_out.headers().unwrap();
        assert!(
            out_headers.iter().any(|h| h == "make_model_id"),
            "Output CSV missing 'make_model_id' column"
        );

        println!("✅ Output file generated successfully: {}", output_path);
    }
}
