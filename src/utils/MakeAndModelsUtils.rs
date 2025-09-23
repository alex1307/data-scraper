use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    hash::Hash,
    path::Path,
};

use csv::{ReaderBuilder, WriterBuilder};

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct LookupTable(pub HashMap<String, Vec<String>>);

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]

pub struct Model {
    pub value: String,
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MakeData {
    pub id: String,
    pub make: String,
    pub model: String,
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CsvModelRecord {
    pub id: String,
    pub make: String,
    pub model: String,
    pub tags: String,
}

// Path: src/utils/MakeAndModelsUtils.rs

pub fn save_make_data_to_csv<P: AsRef<Path>>(
    make_model_map: &HashMap<String, Vec<Model>>,
    path: P,
) -> Result<(), String> {
    let mut wtr = WriterBuilder::new()
        .delimiter(b';')
        .from_path(path)
        .map_err(|e| format!("Failed to create CSV writer: {}", e))?;

    wtr.serialize(("id", "make", "model", "tags"))
        .map_err(|e| format!("Failed to write header: {}", e))?;
    let mut sorted_make_models: Vec<_> = make_model_map.iter().collect();
    sorted_make_models.sort_by_key(|(make, _)| make.to_lowercase());

    let mut make_id_counter = 1;
    for (make, models) in sorted_make_models {
        let make_id = format!("{:03}", make_id_counter);
        make_id_counter += 1;

        let mut sorted_models = models.clone();
        sorted_models.sort_by_key(|model| model.value.to_lowercase());

        for (idx, model) in sorted_models.iter().enumerate() {
            let model_id = format!("{:03}", idx + 1);
            let full_id = format!("{}{}", make_id, model_id);
            let tags = model.tags.join(" ");

            let record = CsvModelRecord {
                id: full_id,
                make: make.clone(),
                model: model.value.clone(),
                tags,
            };

            wtr.serialize(record)
                .map_err(|e| format!("Failed to write record: {}", e))?;
        }
    }

    wtr.flush()
        .map_err(|e| format!("Failed to flush writer: {}", e))?;
    Ok(())
}

pub fn update_make_data(new_data: &HashMap<String, Vec<Model>>, path: &str) {
    let mut existing_map: HashMap<String, Vec<Model>> = if Path::new(path).exists() {
        let data = fs::read_to_string(path).unwrap();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        HashMap::new()
    };

    for (make, new_models) in new_data {
        let entry = existing_map.entry(make.clone()).or_insert_with(Vec::new);
        let existing_values: HashSet<String> = entry.iter().map(|m| m.value.clone()).collect();
        for model in new_models {
            if !existing_values.contains(&model.value) {
                entry.push(model.clone());
            }
        }
    }

    let updated_json = serde_json::to_string_pretty(&existing_map).unwrap();
    fs::write(path, updated_json).unwrap();
    println!("✅ Updated model lookup table saved to '{}'", path);
}

pub fn build_make_model_map<P: AsRef<Path>>(
    paths: Vec<P>,
    allowed_source: Vec<&str>,
) -> HashMap<String, Vec<Model>> {
    let mut unique_models = HashSet::new();

    for path in paths {
        let file = File::open(&path).expect("Failed to open CSV file");
        let mut rdr = ReaderBuilder::new().delimiter(b';').from_reader(file);

        let headers = rdr.headers().expect("Failed to read headers").clone();
        let make_idx = headers
            .iter()
            .position(|h| h == "make")
            .expect("Missing 'make' column");
        let model_idx = headers
            .iter()
            .position(|h| h == "model")
            .expect("Missing 'model' column");
        let source_idx = headers
            .iter()
            .position(|h| h == "source")
            .expect("Missing 'source' column");

        for result in rdr.records() {
            let record = result.expect("Failed to read record");
            let source = record.get(source_idx).unwrap_or("").trim();
            if !allowed_source.contains(&source) {
                continue;
            }

            let make = record.get(make_idx).unwrap_or("").trim().to_string();
            let model = record.get(model_idx).unwrap_or("").trim().to_string();
            let tags: Vec<String> = tokenize(&model);

            let make_entry = MakeData {
                id: format!("{}{}", make.to_uppercase(), model.to_uppercase()),
                make,
                model,
                tags,
            };

            unique_models.insert(make_entry);
        }
    }

    let mut make_model_map: HashMap<String, Vec<Model>> = HashMap::new();
    for make_entry in unique_models {
        let model_entry = Model {
            value: make_entry.model.clone(),
            tags: make_entry.tags.clone(),
        };

        make_model_map
            .entry(make_entry.make)
            .or_insert_with(Vec::new)
            .push(model_entry);
    }

    make_model_map
}

impl Hash for MakeData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for MakeData {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for MakeData {}

pub fn tokenize(model: &str) -> Vec<String> {
    model
        .split(|c: char| c.is_whitespace() || c == '.')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string().trim().to_lowercase())
        .collect()
}

pub fn find_unmatched_models(
    make_model_map: &HashMap<String, Vec<Model>>,
    new_data: &HashMap<String, Vec<Model>>,
) -> HashMap<String, Vec<Model>> {
    let mut unmatched_models = HashMap::new();

    for (make, new_models) in new_data {
        if let Some(existing_models) = make_model_map.get(make) {
            let existing_set: HashSet<_> = existing_models.iter().collect();
            let new_set: HashSet<_> = new_models.iter().collect();

            let unmatched: Vec<_> = new_set
                .difference(&existing_set)
                .map(|model| (*model).clone())
                .collect();

            if !unmatched.is_empty() {
                unmatched_models.insert(make.clone(), unmatched);
            }
        } else {
            unmatched_models.insert(make.clone(), new_models.iter().cloned().collect());
        }
    }

    unmatched_models
}

impl PartialEq for Model {
    fn eq(&self, other: &Self) -> bool {
        if other.value.is_empty() {
            return false;
        }
        if self.value.is_empty() {
            return false;
        }
        if self.value == other.value {
            return true;
        }
        self.value.to_lowercase().trim() == other.value.to_lowercase().trim()
    }
}

impl Eq for Model {}
impl Hash for Model {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.to_lowercase().hash(state);
    }
}
