use std::collections::HashMap;
use super::mobile_utils::extract_ascii_latin;
#[derive(Debug, Clone, PartialEq)]
pub struct SearchRequest {
    pub make: String,
    pub model: String,
    pub engine: String,
    pub transmission: String,
    pub from_year: u16,
    pub to_year: u16,
    pub lpg: bool,
    pub four_wheel_drive: bool,
    pub registration_number: bool,
}   
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResponse {
    pub slink: String,
    pub links: Vec<String>,
    pub make: String,
    pub model: String,
    pub number_of_vehicle: u16,
    pub min_price: f32,
    pub max_price: f32,
    pub sum_of_prices: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetaHeader{
    pub make: String,
    pub model: String,
    pub total_number: u16,
    pub min_price: u32,
    pub max_price: u32,
}



impl SearchRequest {
    pub fn new(make: String, model: String) -> Self {
        SearchRequest {
            make,
            model,
            engine: String::new(),
            transmission: String::new(),
            from_year: 0,
            to_year: 0,
            lpg: false,
            four_wheel_drive: false,
            registration_number: false,
        }
    }

    pub fn set_engine(&mut self, engine: String) {
        self.engine = engine;
    }

    pub fn set_transmission(&mut self, transmission: String) {
        self.transmission = transmission;
    }

    pub fn set_from_year(&mut self, from_year: u16) {
        self.from_year = from_year;
    }

    pub fn set_to_year(&mut self, to_year: u16) {
        self.to_year = to_year;
    }

    pub fn to_form_data(&self) -> HashMap<&str, String> {
        let mut form_data = HashMap::new();
        form_data.insert("rub_pub_save", 1.to_string());
        form_data.insert("act", 3.to_string());
        form_data.insert("rub", 1.to_string());
        form_data.insert("f5", self.make.clone());
        form_data.insert("f6", self.model.clone());
        
        if self.from_year >1950 {
            form_data.insert("f10", self.from_year.to_string());
        }

        if self.to_year >1950 {
            form_data.insert("f11", self.to_year.to_string());
        }

        if !self.engine.is_empty() {
            form_data.insert("f12", self.engine.clone());
        }

        if !self.transmission.is_empty() {
            form_data.insert("f13", self.transmission.clone());
        }

        if self.four_wheel_drive {
            form_data.insert("88", 1.to_string());
        }

        if self.lpg {
            form_data.insert("92", 1.to_string());
        }

        if self.registration_number {
            form_data.insert("102", 1.to_string());
        }

        return form_data;
    }
}

impl MetaHeader{
    pub fn from_string(raw: &str) -> Self {
        let meta = extract_ascii_latin(raw);
        let re = regex::Regex::new(r" {2,}").unwrap();
        let split: Vec<&str> = re.split(&meta.trim()).collect();
        if split.len() <= 4 {
            return MetaHeader {
                make: "".to_string(),
                model: "".to_string(),
                min_price: 0,
                max_price: 0,
                total_number: 0,
            };
        }

        let make_model: Vec<&str> = split[0].split_whitespace().collect();

        let (make, model) = if make_model.len() == 1 {
            (make_model[0], "")
        } else {
            (make_model[0], make_model[1])
        };

        let min = split[1].replace(" ", "").parse::<u32>().unwrap_or(0);
        let max = split[2].replace(" ", "").parse::<u32>().unwrap_or(0);
        let total_number = split[3].replace(" ", "").parse::<u16>().unwrap_or(0);

        MetaHeader {
            make: make.to_string(),
            model: model.to_string(),
            min_price: min,
            max_price: max,
            total_number: total_number,
        }
    }
}