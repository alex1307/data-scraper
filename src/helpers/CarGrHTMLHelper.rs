use std::{collections::HashMap, str::FromStr};

use chrono::Duration;
use log::info;
use scraper::{Html, Selector};
use serde::Deserialize;

use crate::{
    config::Equipment::get_equipment_as_u64,
    helpers::{
        CURRENCY_KEY, DEALER_KEY, ENGINE_KEY, EQUIPMENT_KEY, GEARBOX_KEY, LOCATION_KEY, MAKE_KEY,
        MILEAGE_KEY, MODEL_KEY, PHONE_KEY, POWER_KEY, PRICE_KEY, PUBLISHED_ON_KEY, TOP_KEY,
        VIEW_COUNT_KEY, YEAR_KEY,
    },
    model::{
        enums::{Currency, Engine, Gearbox},
        VehicleRecord::MobileRecord,
    },
    CREATED_ON, DATE_FORMAT, NOW,
};

#[derive(Debug, Deserialize, Clone)]
struct KeyValue {
    name: String,
}

pub fn vehicle_data(html_page: &str) -> HashMap<String, String> {
    let mut vehicle = HashMap::new();
    let specification = get_specification(html_page);
    let dealer = get_dealer_data(html_page);
    let equipment = get_equipment(html_page);
    vehicle.insert(EQUIPMENT_KEY.to_owned(), equipment.to_string());
    vehicle.extend(specification);
    vehicle.extend(dealer);
    vehicle
}

pub fn get_specification(html_page: &str) -> HashMap<String, String> {
    let mut data = HashMap::new();
    let document = Html::parse_document(html_page);
    let row_selector = Selector::parse("tr.c-table-row").unwrap();
    let td_selector = Selector::parse("td").unwrap();
    for row in document.select(&row_selector) {
        let tds = row.select(&td_selector).collect::<Vec<_>>();
        if tds.len() >= 2 {
            let key = tds[0].text().collect::<String>();
            let value = tds[1].text().collect::<String>();
            let key = key
                .trim()
                .replace([':', ' ', '\t', '\n', '\r'], "")
                .trim()
                .to_lowercase();
            let value = value.replace(['\n', '\r'], "").trim().to_string();
            if PRICE_KEY.to_owned() == key {
                let value = value
                    .chars()
                    .filter(|&c| c.is_numeric())
                    .collect::<String>()
                    .parse::<u32>()
                    .unwrap_or(0);
                data.insert(PRICE_KEY.to_owned(), value.to_string());
            } else if *"fueltype" == key {
                data.insert(ENGINE_KEY.to_owned(), value.to_string());
            } else if POWER_KEY.to_owned() == key {
                let value = value
                    .chars()
                    .filter(|&c| c.is_numeric())
                    .collect::<String>()
                    .parse::<u32>()
                    .unwrap_or(0);
                data.insert(POWER_KEY.to_owned(), value.to_string());
            } else if MILEAGE_KEY.to_owned() == key {
                let value = value
                    .chars()
                    .filter(|&c| c.is_numeric())
                    .collect::<String>()
                    .parse::<u32>()
                    .unwrap_or(0);
                data.insert(MILEAGE_KEY.to_owned(), value.to_string());
            } else if "registration" == key {
                let value = if value.contains('/') {
                    let year = value.split('/').collect::<Vec<_>>()[1];
                    year.chars()
                        .filter(|&c| c.is_numeric())
                        .collect::<String>()
                        .parse::<u32>()
                        .unwrap_or(0)
                } else {
                    value
                        .chars()
                        .filter(|&c| c.is_numeric())
                        .collect::<String>()
                        .parse::<u32>()
                        .unwrap_or(0)
                };
                data.insert(YEAR_KEY.to_owned(), value.to_string());
            } else if *"transmission" == key {
                data.insert(GEARBOX_KEY.to_owned(), value.to_string());
            } else if "engine" == key {
                let value = value
                    .chars()
                    .filter(|&c| c.is_numeric())
                    .collect::<String>()
                    .parse::<u32>()
                    .unwrap_or(0);
                data.insert("cc".to_owned(), value.to_string());
            } else if "make-model" == key {
                data.insert("full".to_owned(), value.to_owned());
                let value = value.split(' ').collect::<Vec<_>>();
                if value.len() >= 2 {
                    let make = value[0].to_owned();
                    let model = value[1].to_owned();
                    data.insert(MAKE_KEY.to_owned(), make);
                    data.insert(MODEL_KEY.to_owned(), model);
                }
            } else if "views" == key {
                let value = value
                    .chars()
                    .filter(|&c| c.is_numeric())
                    .collect::<String>()
                    .parse::<u32>()
                    .unwrap_or(0);
                data.insert(VIEW_COUNT_KEY.to_owned(), value.to_string());
            } else if "telephone" == key {
                let value = value
                    .chars()
                    .filter(|&c| !c.is_whitespace())
                    .collect::<String>();

                data.insert(PHONE_KEY.to_owned(), value.to_owned());
            } else if "modified" == key {
                let modified = value.split_ascii_whitespace().collect::<Vec<_>>();
                let duration = match modified[1] {
                    "hours" => {
                        let hours = -modified[0].parse::<i64>().unwrap_or(0);
                        Duration::hours(hours)
                    }
                    "days" => {
                        let days = -modified[0].parse::<i64>().unwrap_or(0);
                        Duration::days(days)
                    }
                    "months" => {
                        let months = -modified[0].parse::<i64>().unwrap_or(0);
                        Duration::days(months * 30)
                    }
                    _ => Duration::hours(0),
                };
                match NOW.checked_add_signed(duration) {
                    Some(date) => {
                        let date = date.format(DATE_FORMAT).to_string();
                        data.insert(PUBLISHED_ON_KEY.to_owned(), date);
                    }
                    None => {
                        data.insert(CREATED_ON.to_owned(), NOW.format(DATE_FORMAT).to_string());
                    }
                }
            } else {
                data.insert(key, value);
            }
        }
    }
    data.insert(CURRENCY_KEY.to_owned(), "EUR".to_owned());
    if html_page.contains("Promoted") {
        data.insert(TOP_KEY.to_owned(), "true".to_owned());
    }
    data
}

pub fn get_dealer_data(html_page: &str) -> HashMap<String, String> {
    let document = Html::parse_document(html_page);
    let selector = Selector::parse("div.main-seller-info a").unwrap();
    let mut data = HashMap::new();
    for element in document.select(&selector) {
        match element.value().attr("href") {
            Some(href) => {
                data.insert("dealer_url".to_owned(), href.to_owned());
            }
            None => continue,
        };

        match element.value().attr("title") {
            Some(title) => {
                data.insert(DEALER_KEY.to_owned(), title.to_owned());
            }
            None => continue,
        };
        break;
    }

    let selector = Selector::parse("div.main-seller-info span").unwrap();
    if let Some(element) = document.select(&selector).next() {
        let text = element.text().collect::<Vec<_>>().join(" ");
        let text = text.trim().replace('\n', "");
        data.insert(LOCATION_KEY.to_owned(), text.trim().to_owned());
    }
    data
}

pub fn get_equipment(html_page: &str) -> u64 {
    if let Some(start_index) = html_page.find("<script>window.__NUXT__=(function(a,b,c,d,") {
        let sub = &html_page[start_index..];
        if let Some(end_index) = sub.find("</script>") {
            let sub = &sub[..end_index];
            if let Some(extras_start_index) = sub.find("extras:[") {
                let extralist = &sub[extras_start_index..];
                if let Some(extras_end_index) = extralist.find("],") {
                    let extralist = &extralist[7..extras_end_index + 1];
                    let json_text = extralist
                        .replace("key:", "\"key\":")
                        .replace("value:", "\"value\":")
                        .replace("name:", "\"name\":");
                    let values = serde_json::from_str::<Vec<KeyValue>>(json_text.trim()).unwrap();
                    let equipment_list = values.into_iter().map(|v| v.name).collect::<Vec<_>>();
                    let equipment_id = get_equipment_as_u64(equipment_list);
                    return equipment_id;
                }
            }
        }
    }
    0
}

pub fn get_listed_links(source: &str) -> Vec<String> {
    let mut links = vec![];
    let document = Html::parse_document(source);
    let vehicle_selector = Selector::parse("li > div.search-row > a").unwrap();
    for element in document.select(&vehicle_selector) {
        let href = match element.value().attr("href") {
            Some(href) => href,
            None => continue,
        };
        links.push(href.to_owned());
    }
    links
}

pub fn get_total_number(source: &str) -> u32 {
    let document = Html::parse_document(source);
    let selector = Selector::parse("li.c-breadcrumb").unwrap();
    let mut total_number = 0;
    for element in document.select(&selector) {
        let text = element.text().collect::<Vec<_>>().join(" ");
        if text.contains("Results") {
            info!("Found element text: {}", text);
            total_number = text
                .chars()
                .filter(|&c| c.is_numeric())
                .collect::<String>()
                .parse::<u32>()
                .unwrap_or(0);
            break;
        }
    }
    total_number
}

pub fn process_listed_links(source: &str) -> Vec<MobileRecord> {
    let html = Html::parse_document(source);
    let div_selector = Selector::parse(r#"div.overlay-content-container"#).unwrap();
    let li_selector = Selector::parse("li").unwrap();
    let location_selector = Selector::parse("span.tw-text-grey-700").unwrap();
    let fuel_selector = Selector::parse("span.key-feature[title='Fuel type']").unwrap();
    let transimission_selector = Selector::parse("span.key-feature[title='Transmission']").unwrap();
    let power_selector = Selector::parse("span.key-feature[title='Power']").unwrap();
    let engine_selector = Selector::parse("span.key-feature[title='Engine']").unwrap();
    let mieage_selector = Selector::parse("span.key-feature[title='Mileage']").unwrap();
    let registration_selector = Selector::parse("span.key-feature[title='Registration']").unwrap();
    let current_price_selector =
        Selector::parse("div.price-tag.current-price span > span:first-child").unwrap();
    let title_selector = Selector::parse("h2.title.title").unwrap();

    // let old_price_selector = Selector::parse("div.price-tag.old-price").unwrap();
    let href_selector = Selector::parse("a.row-anchor").unwrap();
    let mut basic_info = vec![];
    for div in html.select(&div_selector) {
        // Now iterate over li elements within the div
        for li in div.select(&li_selector) {
            let mut info = MobileRecord::default();

            for element in li.select(&href_selector) {
                // Access the href attribute
                if let Some(href) = element.value().attr("href") {
                    let id = href
                        .to_owned()
                        .chars()
                        .filter(|&c| c.is_numeric())
                        .collect::<String>();
                    info.id = id;
                }
            }
            for span in li.select(&title_selector) {
                let text = span.text().collect::<Vec<_>>().join(" ");
                let text = text.trim().replace('\n', "");
                info.title = text.clone();
                if text.contains('\'') {
                    let text = text.split('\'').collect::<Vec<_>>();
                    let make_model = text[0].to_owned();
                    info!("make_model: {}", make_model);
                    info.make = make_model.split(' ').collect::<Vec<_>>()[0].to_owned();
                    info.model = make_model.split(' ').collect::<Vec<_>>()[1].to_owned();
                }
            }
            for span in li.select(&location_selector) {
                let text = span.text().collect::<Vec<_>>().join(" ");
                info!("location: {}", text.trim().replace('\n', ""));
            }
            for span in li.select(&fuel_selector) {
                let text = span.text().collect::<Vec<_>>().join(" ");
                let text = text.trim().replace('\n', "");
                if let Ok(engine) = Engine::from_str(&text) {
                    info.engine = engine;
                }
            }
            for span in li.select(&transimission_selector) {
                let text = span.text().collect::<Vec<_>>().join(" ");
                let text = text.trim().replace('\n', "");
                if let Ok(gearbox) = Gearbox::from_str(&text) {
                    info.gearbox = gearbox;
                }
            }

            for span in li.select(&power_selector) {
                let text = span.text().collect::<Vec<_>>().join(" ");
                let text = text.trim().replace('\n', "");
                let power = text.chars().filter(|&c| c.is_numeric()).collect::<String>();
                info.power = power.parse::<u32>().unwrap_or(0);
            }

            for span in li.select(&engine_selector) {
                let text = span.text().collect::<Vec<_>>().join(" ");
                let text = text.trim().replace('\n', "");
                let cc = text.chars().filter(|&c| c.is_numeric()).collect::<String>();
                info.cc = cc.parse::<u32>().unwrap_or(0);
            }

            for span in li.select(&mieage_selector) {
                let text = span.text().collect::<Vec<_>>().join(" ");
                let text = text.trim().replace('\n', "");
                let millage = text.chars().filter(|&c| c.is_numeric()).collect::<String>();
                info.mileage = millage.parse::<u32>().unwrap_or(0);
            }

            for span in li.select(&registration_selector) {
                let text = span.text().collect::<Vec<_>>().join(" ");
                let text = text.trim().replace('\n', "");
                if text.contains('/') {
                    let year = text.split('/').collect::<Vec<_>>();
                    info.year = year[1]
                        .chars()
                        .filter(|&c| c.is_numeric())
                        .collect::<String>()
                        .parse::<u16>()
                        .unwrap_or(0);
                    info.month = year[0]
                        .chars()
                        .filter(|&c| c.is_numeric())
                        .collect::<String>()
                        .parse::<u16>()
                        .unwrap_or(0);
                } else {
                    let year = text.chars().filter(|&c| c.is_numeric()).collect::<String>();
                    info.year = year.parse::<u16>().unwrap_or(0);
                }
            }

            for p in li.select(&current_price_selector) {
                let text = p.text().collect::<Vec<_>>().join(" ");
                let text = text.trim().replace('\n', "");
                let price = text.chars().filter(|&c| c.is_numeric()).collect::<String>();
                info.currency = Currency::EUR;
                info.price = price.parse::<u32>().unwrap_or(0);
            }
            basic_info.push(info);

            // for p in li.select(&old_price_selector) {
            //     let text = p.text().collect::<Vec<_>>().join(" ");
            //     let text = text.trim().replace('\n', "");
            // }
        }
    }
    basic_info
}

#[cfg(test)]
mod car_gr_test_suit {
    use std::collections::HashMap;

    use crate::{
        helpers::CarGrHTMLHelper::{get_listed_links, get_total_number},
        scraper::Traits::Scraper,
        utils::helpers::configure_log4rs,
        LOG_CONFIG,
    };

    #[tokio::test]
    async fn get_listes_vehicles_test() {
        configure_log4rs(&LOG_CONFIG);
        let url = "https://www.car.gr/classifieds/cars/?";
        let mut params = HashMap::new();
        params.insert("lang".to_owned(), "en".to_owned());
        params.insert("fs".to_owned(), "1".to_owned());
        params.insert("category".to_owned(), "15001".to_owned());
        params.insert("price-from".to_owned(), "25000".to_owned());
        params.insert("price-to".to_owned(), "30000".to_owned());
        params.insert("registration-from".to_owned(), "2010".to_owned());
        params.insert("registration-to".to_owned(), "2011".to_owned());
        //params.insert("created".to_owned(), ">1".to_owned());
        let scraper = Scraper::new(url, "pg".to_owned(), 250);
        let url = scraper.search_url(None, params, 1);
        let html = scraper.html_search(&url, None).await.unwrap();
        //info!("html: {}", html);
        let data = get_listed_links(&html);
        assert_eq!(data.len(), 24);
        let total_number = get_total_number(&html);
        assert_eq!(total_number, 125);
    }
}
