use std::collections::HashMap;

use chrono::Duration;
use log::info;
use scraper::{Html, Selector};
use serde::Deserialize;

use crate::{
    config::equipment::{get_equipment_as_u64, CAR_GR_EQUIPMENT},
    scraper::{
        CURRENCY_KEY, DEALER_KEY, ENGINE_KEY, EQUIPMENT_KEY, GEARBOX_KEY, LOCATION_KEY, MAKE_KEY,
        MILEAGE_KEY, MODEL_KEY, PHONE_KEY, POWER_KEY, PRICE_KEY, PUBLISHED_ON_KEY, TOP_KEY,
        VIEW_COUNT_KEY, YEAR_KEY,
    },
    CREATED_ON, DATE_FORMAT, NOW,
};

#[derive(Debug, Deserialize, Clone)]
struct KeyValue {
    key: i32,
    value: String,
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
    let document = Html::parse_document(&html_page);
    let row_selector = Selector::parse("tr.c-table-row").unwrap();
    let td_selector = Selector::parse("td").unwrap();
    for row in document.select(&row_selector) {
        let tds = row.select(&td_selector).collect::<Vec<_>>();
        if tds.len() >= 2 {
            let key = tds[0].text().collect::<String>();
            let value = tds[1].text().collect::<String>();
            let key = key
                .trim()
                .replace(":", "")
                .replace(" ", "")
                .replace("\t", "")
                .replace("\n", "")
                .replace("\r", "")
                .trim()
                .to_lowercase();
            let value = value.replace("\n", "").replace("\r", "").trim().to_string();
            if PRICE_KEY.to_owned() == key {
                let value = value
                    .chars()
                    .filter(|&c| c.is_numeric())
                    .collect::<String>()
                    .parse::<u32>()
                    .unwrap_or(0);
                data.insert(PRICE_KEY.to_owned(), value.to_string());
            } else if "fueltype".to_owned() == key {
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
            } else if "transmission".to_owned() == key {
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
                info!("key: {}, value: {}", key, value.trim())
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
    let document = Html::parse_document(&html_page);
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
    for element in document.select(&selector) {
        let text = element.text().collect::<Vec<_>>().join(" ");
        let trimmed = text.replace("\n", "").replace("\r", "");
        data.insert(LOCATION_KEY.to_owned(), trimmed.trim().to_owned());
        break;
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
                    let equipment_id = get_equipment_as_u64(equipment_list, &CAR_GR_EQUIPMENT);
                    return equipment_id;
                }
            }
        }
    } else {
        info!("Not found");
    }
    0
}

pub fn get_listed_links(source: &str) -> Vec<String> {
    let mut links = vec![];
    let document = Html::parse_document(&source);
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
    let document = Html::parse_document(&source);
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

#[cfg(test)]
mod car_gr_test_suit {
    use std::collections::HashMap;

    use log::info;
    use scraper::{Html, Selector};

    use crate::{
        helpers::CarGrHTMLHelper::{
            get_dealer_data, get_equipment, get_listed_links, get_specification, get_total_number,
            vehicle_data,
        },
        model::records::MobileRecord,
        scraper::{CarGrScraper::CarGrScraper, ScraperTrait::Scraper},
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
        assert_eq!(total_number, 122);
    }

    #[tokio::test]
    async fn get_vehicle_test() {
        configure_log4rs(&LOG_CONFIG);
        let url = "https://www.car.gr/classifieds/cars/view/319951193-mercedes-benz-e-350?lang=en";
        let scraper = CarGrScraper::new(url, 250);
        let page = scraper.parent.html_search(url, None).await.unwrap();
        info!("page: {}", page.as_bytes().len());
        //info!("page: {}", page);
        let selector = Selector::parse("div.tw-text-base").unwrap();
        let document = Html::parse_document(&page);
        for element in document.select(&selector) {
            let text = element.text().collect::<Vec<_>>().join(" ");
            if text.contains("Registration") {
                text.split("\n").for_each(|line| {
                    info!("line: {}", line.trim().replace(' ', ""));
                    if line.contains('/') {
                        let millage = line.split('/').collect::<Vec<_>>()[1];
                        let millage = millage
                            .chars()
                            .filter(|&c| c.is_numeric())
                            .collect::<String>()
                            .parse::<u32>()
                            .unwrap_or(0);
                        info!("year: {}", millage);
                    } else {
                        let value = line
                            .chars()
                            .filter(|&c| c.is_numeric())
                            .collect::<String>()
                            .parse::<u32>()
                            .unwrap_or(0);
                        info!("year: {}", value);
                    }
                });
            }

            if text.contains("Fuel type") {
                text.split("\n").for_each(|line| {
                    info!("line: {}", line.trim().replace(' ', ""));
                });
            }

            if text.contains("Engine") {
                text.split("\n").for_each(|line| {
                    info!("line: {}", line.trim().replace(' ', ""));
                    if line.contains("cc") {
                        let millage = line.split("cc").collect::<Vec<_>>()[0];
                        let cc = millage
                            .chars()
                            .filter(|&c| c.is_numeric())
                            .collect::<String>()
                            .parse::<u32>()
                            .unwrap_or(0);
                        info!("cc: {}", cc);
                    }
                });
            }

            if text.contains("Transmission") {
                text.split("\n").for_each(|line| {
                    info!("line: {}", line.trim().replace(' ', ""));
                });
            }

            if text.contains("Power") {
                text.split("\n").for_each(|line| {
                    info!("line: {}", line.trim().replace(' ', ""));
                    if line.contains("bhp") {
                        let millage = line.split("bhp").collect::<Vec<_>>()[0];
                        let millage = millage
                            .chars()
                            .filter(|&c| c.is_numeric())
                            .collect::<String>()
                            .parse::<u32>()
                            .unwrap_or(0);
                        info!("power: {}", millage);
                    }
                });
            }

            if text.contains("Mileage") {
                text.split("\n").for_each(|line| {
                    let line = line.trim().replace(' ', "");
                    info!("line: {}", line);
                    if line.contains("km") {
                        let millage = line.split("km").collect::<Vec<_>>()[0];
                        let millage = millage
                            .chars()
                            .filter(|&c| c.is_numeric())
                            .collect::<String>()
                            .parse::<u32>()
                            .unwrap_or(0);
                        info!("millage: {}", millage);
                    }
                });
            }
        }
    }

    #[tokio::test]
    async fn get_specification_test() {
        configure_log4rs(&LOG_CONFIG);
        let url = "https://www.car.gr/classifieds/cars/view/319951193-mercedes-benz-e-350?lang=en";
        let scraper = CarGrScraper::new(url, 250);
        let page = scraper.parent.html_search(url, None).await.unwrap();
        let data = get_specification(&page);
        info!("data: {:?}", data);
    }

    #[tokio::test]
    async fn get_extras_test() {
        configure_log4rs(&LOG_CONFIG);
        let url = "https://www.car.gr/classifieds/cars/view/338681033-land-rover-range-rover";
        let scraper = CarGrScraper::new(url, 250);
        let page = scraper.parent.html_search(url, None).await.unwrap();
        let equipment = get_equipment(&page);
        assert!(equipment > 0);
        info!("equipment: {}", equipment);
    }

    #[tokio::test]
    async fn get_dealer_test() {
        configure_log4rs(&LOG_CONFIG);
        let url = "https://www.car.gr/classifieds/cars/view/319951193-mercedes-benz-e-350?lang=en";
        let scraper = CarGrScraper::new(url, 250);
        let page = scraper.parent.html_search(url, None).await.unwrap();
        let data = get_dealer_data(&page);
        assert!(!data.is_empty());
        assert_eq!(data.len(), 3);
    }

    #[tokio::test]
    async fn get_private_seller_test() {
        configure_log4rs(&LOG_CONFIG);
        let url =
            "https://www.car.gr/classifieds/cars/view/338681033-land-rover-range-rover?lang=en";
        let scraper = CarGrScraper::new(url, 250);
        let page = scraper.parent.html_search(url, None).await.unwrap();
        let data = get_dealer_data(&page);
        assert!(!data.is_empty());
        assert_eq!(data.len(), 2);
        info!("data: {:?}", data);
    }

    #[tokio::test]
    async fn vehicle_data_test() {
        configure_log4rs(&LOG_CONFIG);
        let url =
            "https://www.car.gr/classifieds/cars/view/338681033-land-rover-range-rover?lang=en";
        let scraper = CarGrScraper::new(url, 250);
        let page = scraper.parent.html_search(url, None).await.unwrap();
        let data = vehicle_data(&page);
        //info!("land-rover: {:?}", data);
        let record = MobileRecord::from(data);
        info!("record: {:?}", record);
        let url = "https://www.car.gr/classifieds/cars/view/319951193-mercedes-benz-e-350?lang=en";
        let scraper = CarGrScraper::new(url, 250);
        let page = scraper.parent.html_search(url, None).await.unwrap();
        let data = vehicle_data(&page);
        // info!("mercedes: {:?}", data);
        let record = MobileRecord::from(data);
        info!("record: {:?}", record);
    }
}
