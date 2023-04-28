use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Request {
    pub act: u8,
    pub make: String,
    pub model: String,
    pub engine: String,
    pub transmission: String,
    pub from_year: u16,
    pub to_year: u16,
    pub lpg: bool,
    pub four_wheel_drive: bool,
    pub registration_number: bool,
    pub latest: bool,
    pub sold: bool,
}

impl Request {
    pub fn new(make: String, model: String) -> Self {
        Request {
            make,
            model,
            engine: String::new(),
            transmission: String::new(),
            from_year: 0,
            to_year: 0,
            lpg: false,
            four_wheel_drive: false,
            registration_number: false,
            act: 3,
            latest: false,
            sold: false,
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

    pub fn set_sold(&mut self, sold: bool) {
        self.sold = sold;
    }

    pub fn set_latest(&mut self, latest: bool) {
        self.latest = latest;
    }

    pub fn set_act(&mut self, act: u8) {
        self.act = act;
    }

    pub fn to_form_data(&self) -> HashMap<String, String> {
        let mut form_data = HashMap::new();
        form_data.insert("rub_pub_save".to_string(), 1.to_string());
        form_data.insert("rub".to_string(), 1.to_string());
        form_data.insert("act".to_string(), self.act.to_string());

        form_data.insert("f1".to_string(), 1.to_string());
        form_data.insert("f2".to_string(), 1.to_string());
        form_data.insert("f3".to_string(), 1.to_string());
        form_data.insert("f4".to_string(), 1.to_string());
        form_data.insert("f9".to_string(), "лв.".to_string());
        form_data.insert("f21".to_string(), "01".to_string());
        for i in 39..132 {
            let key = format!("f{}", i).to_string();
            form_data.insert(key.clone(), 0.to_string());
        }
        if !self.make.is_empty() {
            form_data.insert("f5".to_string(), self.make.clone());
        }

        if !self.model.is_empty() {
            form_data.insert("f6".to_string(), self.model.clone());
        }

        if self.from_year > 1950 {
            form_data.insert("f10".to_string(), self.from_year.to_string());
        }

        if self.to_year > 1950 {
            form_data.insert("f11".to_string(), self.to_year.to_string());
        }

        if !self.engine.is_empty() {
            form_data.insert("f12".to_string(), self.engine.clone());
        }

        if !self.transmission.is_empty() {
            form_data.insert("f13".to_string(), self.transmission.clone());
        }

        if self.four_wheel_drive {
            form_data.insert("88".to_string(), 1.to_string());
        }

        if self.lpg {
            form_data.insert("92".to_string(), 1.to_string());
        }

        if self.registration_number {
            form_data.insert("102".to_string(), 1.to_string());
        }

        if self.latest {
            form_data.insert("f20".to_string(), 7.to_string());
        } else {
            form_data.insert("f20".to_string(), 1.to_string());
        }

        if self.sold {
            form_data.insert("f94".to_string(), "1".to_string());
        }
        form_data
    }
}
