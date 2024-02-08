pub mod AutoUncleHelper;
pub mod CarGrHTMLHelper;
pub mod CarsBgHTMLHelper;
pub mod MobileBgHTMLHelper;
pub mod MobileDeHelper;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref ID_KEY: String = "id".to_string();
    pub static ref LOCATION_KEY: String = "location".to_string();
    pub static ref PHONE_KEY: String = "phone".to_string();
    pub static ref PRICE_KEY: String = "price".to_string();
    pub static ref CURRENCY_KEY: String = "currency".to_string();
    pub static ref YEAR_KEY: String = "year".to_string();
    pub static ref MAKE_KEY: String = "make".to_string();
    pub static ref MODEL_KEY: String = "model".to_string();
    pub static ref DEALER_KEY: String = "dealer".to_string();
    pub static ref SOLD_KEY: String = "sold".to_string();
    pub static ref TOP_KEY: String = "top".to_string();
    pub static ref VIP_KEY: String = "vip".to_string();
    pub static ref MILEAGE_KEY: String = "mileage".to_string();
    pub static ref ENGINE_KEY: String = "engine".to_string();
    pub static ref POWER_KEY: String = "power".to_string();
    pub static ref GEARBOX_KEY: String = "gearbox".to_string();
    pub static ref EQUIPMENT_KEY: String = "equipment".to_string();
    pub static ref PUBLISHED_ON_KEY: String = "published_on".to_string();
    pub static ref VIEW_COUNT_KEY: String = "view_count".to_string();
}
