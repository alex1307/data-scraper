use log::info;
use scraper::{Html, Selector};
use serde::Deserialize;

use crate::config::equipment::{get_equipment_as_u64, CAR_GR_EQUIPMENT};

pub fn get_equipment(html_page: &str) -> u64 {
    if let Some(start_index) = html_page.find("<script>window.__NUXT__=(function(a,b,c,d,") {
        info!("start_index: {}", start_index);
        let sub = &html_page[start_index..];
        //info!("sub: {}", sub);
        if let Some(end_index) = sub.find("</script>") {
            info!("end_index: {}", end_index);
            let sub = &sub[..end_index];
            //info!("sub: {}", sub);
            info!("------------------------------------");
            if let Some(extras_start_index) = sub.find("extras:[") {
                let extralist = &sub[extras_start_index..];
                if let Some(extras_end_index) = extralist.find("],") {
                    let extralist = &extralist[7..extras_end_index + 1];
                    info!("extralist: {}", extralist);
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

#[derive(Debug, Deserialize, Clone)]
struct KeyValue {
    key: i32,
    value: String,
    name: String,
}
#[cfg(test)]
mod car_gr_test_suit {
    use std::collections::HashMap;

    use log::info;
    use scraper::{Html, Selector};

    use crate::{
        helpers::CarGrHTMLHelper::{get_equipment, get_listed_links, get_total_number},
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
        let scraper = CarGrScraper::new(url, "pg".to_owned(), 250);
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
        let scraper = CarGrScraper::new(url, "pg".to_owned(), 250);
        let page = scraper.parent.html_search(url, None).await.unwrap();
        info!("page: {}", page.as_bytes().len());
        //info!("page: {}", page);
        let row_selector = Selector::parse("tr.c-table-row").unwrap();
        let td_selector = Selector::parse("td").unwrap();
        let document = Html::parse_document(&page);
        for row in document.select(&row_selector) {
            let tds = row.select(&td_selector).collect::<Vec<_>>();
            if tds.len() >= 2 {
                let make_model = tds[0].text().collect::<String>();
                let model = tds[1].text().collect::<String>();

                info!("[{}] and [{}]", make_model, model);
            }
        }
    }

    #[tokio::test]
    async fn get_extras_test() {
        configure_log4rs(&LOG_CONFIG);
        let url = "https://www.car.gr/classifieds/cars/view/338681033-land-rover-range-rover";
        let scraper = CarGrScraper::new(url, "pg".to_owned(), 250);
        let page = scraper.parent.html_search(url, None).await.unwrap();
        let equipment = get_equipment(&page);
        assert!(equipment > 0);
        info!("equipment: {}", equipment);
    }
}
