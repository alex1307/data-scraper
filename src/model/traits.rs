use super::enums::{Currency, Engine, Gearbox};

pub trait Identity {
    fn get_id(&self) -> String;
}

pub trait URLResource {
    fn get_url(&self) -> String;
}

pub trait Header {
    fn header() -> Vec<&'static str>;
}

pub trait VehicleT {
    fn id(&self) -> String;
    fn source(&self) -> String;
    fn make(&self) -> String;
    fn model(&self) -> String;
    fn title(&self) -> String;
    fn currency(&self) -> Currency;
    fn price(&self) -> u32;
    fn mileage(&self) -> u32;
    fn year(&self) -> u16;
    fn engine(&self) -> Engine;
    fn gearbox(&self) -> Gearbox;
    fn power_ps(&self) -> u32;
    fn power_kw(&self) -> u32;
    fn cc(&self) -> Option<u32>;
    fn url(&self) -> String;
    fn location(&self) -> Option<String>;
    fn seller_name(&self) -> Option<String>;
    fn equipment(&self) -> Option<String>;
    fn seller_url(&self) -> Option<String>;
    fn consumption_fuel(&self) -> Option<f32>;
    fn consumption_kw(&self) -> Option<f32>;
    fn co2(&self) -> Option<u32>;
    fn range(&self) -> Option<u32>;
    fn days_in_sale(&self) -> Option<u32>;
    fn estimated_price(&self) -> Option<u32>;
    fn ranges(&self) -> Option<String>;
    fn rating(&self) -> Option<String>;
    fn thresholds(&self) -> Vec<u32>;
}
