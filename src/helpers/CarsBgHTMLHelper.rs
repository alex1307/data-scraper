use std::{collections::HashMap, thread::sleep, time::Duration, vec};

use chrono::{naive, Local};
use log::{debug, info};
use scraper::{Html, Selector};

use crate::{
    config::Equipment, helpers::PUBLISHED_ON_KEY, model::enums::Engine, CARS_BG_LISTING_URL,
};

use super::{
    MobileBgHTMLHelper::get_pages_async, DEALER_KEY, ENGINE_KEY, EQUIPMENT_KEY, GEARBOX_KEY,
    LOCATION_KEY, MAKE_KEY, MILEAGE_KEY, MODEL_KEY, PHONE_KEY, POWER_KEY, PRICE_KEY, YEAR_KEY,
};
use lazy_static::lazy_static;

lazy_static! {
    static ref PRICE_SELECTOR: Selector = Selector::parse("div.offer-price > strong").unwrap();
    static ref VIEW_COUNT_SELECTOR: Selector = Selector::parse("div#offer_view_count").unwrap();
    static ref MAKE_MODEL_SELECTOR: Selector = Selector::parse("div.text-copy h2").unwrap();
    static ref PHONE_SELECTOR: Selector = Selector::parse("a.a_call_link > div").unwrap();
}

fn search_cars_bg_url(params: &HashMap<String, String>, page: u32) -> String {
    let mut url = CARS_BG_LISTING_URL.to_owned();
    for (key, value) in params.iter() {
        url = format!("{}{}={}&", url, key, value);
    }
    if page == 0 {
        return url.trim_end_matches('&').to_owned();
    }

    url = format!("{}page={}", url, page);
    url
}

async fn list_pages(url: &str) -> u32 {
    let html = get_pages_async(url, false).await.unwrap();
    let document = Html::parse_document(&html);
    let total_number_selector = Selector::parse("span.milestoneNumberTotal").unwrap();
    let element = document.select(&total_number_selector).next().unwrap();
    let total_number = element
        .inner_html()
        .chars()
        .filter(|&c| c.is_numeric())
        .collect::<String>()
        .parse::<i32>()
        .unwrap_or(0);
    info!("totalNumber: {}", total_number);
    let number_of_pages: u32 = ((total_number / 20) + 1).try_into().unwrap();
    number_of_pages
}

pub async fn search_cars_bg(params: HashMap<String, String>) -> Vec<HashMap<String, String>> {
    let url = search_cars_bg_url(&params, 1);
    let number_of_pages = list_pages(&url).await;
    let html = get_pages_async(&url, false).await.unwrap();
    if number_of_pages == 1 {
        read_listing(&html, false)
    } else {
        let mut result = read_listing(&html, false);
        for i in 2..number_of_pages {
            sleep(Duration::from_secs(1));
            info!("page: {}", i);
            let url = search_cars_bg_url(&params, i);
            let html = get_pages_async(&url, false).await.unwrap();
            result.extend(read_listing(&html, false));
        }
        result
    }
}

fn read_listing(html: &str, parse: bool) -> Vec<HashMap<String, String>> {
    let mut result = vec![];
    let document = Html::parse_document(html);
    let selector = Selector::parse("div.mdc-card__primary-action").unwrap();
    for element in document.select(&selector) {
        let mut map = HashMap::new();

        let html_fragment = Html::parse_fragment(element.inner_html().as_str());
        let selector = Selector::parse("a").unwrap();
        for e in html_fragment.select(&selector) {
            let href = e.value().attr("href").unwrap();
            let id = href.split("/offer/").last().unwrap();
            map.insert("id".to_owned(), id.to_owned());
        }

        if !parse {
            result.push(map);
            continue;
        }

        let h6_selector = Selector::parse("h6").unwrap();
        let mut fragment_counter = 1;
        for v in html_fragment.select(&h6_selector) {
            let holder = v
                .inner_html()
                .split_ascii_whitespace()
                .map(|s| s.to_owned())
                .collect::<Vec<String>>();
            if fragment_counter == 1 {
                let date = holder[0].trim_end_matches(',');
                if r#"днес"# == date {
                    let today = Local::now().date_naive();
                    map.insert(PUBLISHED_ON_KEY.to_owned(), today.to_string());
                } else if r#"вчера"# == date {
                    let yesterday = Local::now().date_naive() - chrono::Duration::days(1);
                    map.insert(PUBLISHED_ON_KEY.to_owned(), yesterday.to_string());
                } else {
                    let published_on = naive::NaiveDate::parse_from_str(date, "%d.%m.%y").unwrap();
                    map.insert(PUBLISHED_ON_KEY.to_owned(), published_on.to_string());
                }
            } else if fragment_counter == 2 {
                let price = holder[0]
                    .chars()
                    .filter(|&c| c.is_numeric())
                    .collect::<String>()
                    .parse::<i32>()
                    .unwrap_or(0);
                map.insert(PRICE_KEY.to_owned(), price.to_string());
                break;
            }
            fragment_counter += 1;
        }

        let card_2nd_line_selector = Selector::parse("div.card__secondary").unwrap();
        if let Some(v) = html_fragment.select(&card_2nd_line_selector).next() {
            let holder = v
                .inner_html()
                .split_ascii_whitespace()
                .map(|s| s.trim_end_matches(',').to_owned())
                .collect::<Vec<String>>();
            let year = holder[0]
                .chars()
                .filter(|&c| c.is_numeric())
                .collect::<String>()
                .parse::<i32>()
                .unwrap_or(0);
            let millage = holder[2]
                .chars()
                .filter(|&c| c.is_numeric())
                .collect::<String>()
                .parse::<i32>()
                .unwrap_or(0);

            if let Ok(engine) = <Engine as std::str::FromStr>::from_str(&holder[1]) {
                map.insert(ENGINE_KEY.to_owned(), engine.to_string());
            }
            map.insert(YEAR_KEY.to_owned(), year.to_string());
            map.insert(MILEAGE_KEY.to_owned(), millage.to_string());
            break;
        }

        let card_footer_selector = Selector::parse("div.card__footer").unwrap();
        for v in html_fragment.select(&card_footer_selector) {
            let holder = v
                .inner_html()
                .replace('\n', "")
                .split(',')
                .map(|s| s.trim().to_owned())
                .collect::<Vec<String>>();
            let location = holder[1].to_owned();
            map.insert(LOCATION_KEY.to_owned(), location);
        }
        result.push(map);
    }
    result
}

pub async fn get_ids(url: String) -> Result<Vec<String>, reqwest::Error> {
    let html = get_pages_async(&url, false).await.unwrap();
    let document = Html::parse_document(&html);
    let selector = Selector::parse("div.offer-item").unwrap();
    let mut ids = vec![];
    for element in document.select(&selector) {
        if let Some(id) = element.value().attr("data-id") {
            ids.push(id.to_owned());
        }
    }
    Ok(ids)
}

pub fn read_carsbg_details(html: String) -> HashMap<String, String> {
    let mut result = HashMap::new();
    if html.contains(r#"Частно лице"#) {
        result.insert(DEALER_KEY.to_owned(), "false".to_owned());
    } else {
        result.insert(DEALER_KEY.to_owned(), "true".to_owned());
    }
    let document = Html::parse_document(&html);
    if let Some(el) = document.select(&PRICE_SELECTOR).next() {
        let price = el
            .inner_html()
            .chars()
            .filter(|&c| c.is_numeric())
            .collect::<String>()
            .parse::<i32>()
            .unwrap_or(0);
        result.insert(PRICE_KEY.to_owned(), price.to_string());
    } else {
        result.clear();
        return result;
    }

    if let Some(phone_txt) = document.select(&PHONE_SELECTOR).next() {
        let phone = phone_txt.inner_html();
        result.insert(
            PHONE_KEY.to_owned(),
            phone.replace('\n', "").trim().to_string(),
        );
    }
    let make_model = document.select(&MAKE_MODEL_SELECTOR).next();
    let make_model = match make_model {
        None => {
            result.clear();
            return result;
        }
        Some(mm) => mm
            .inner_html()
            .split_ascii_whitespace()
            .map(|s| s.to_owned())
            .collect::<Vec<String>>(),
    };
    if make_model.len() > 1 {
        result.insert(MAKE_KEY.to_owned(), make_model[0].to_owned());
        result.insert(MODEL_KEY.to_owned(), make_model[1].to_owned());
    } else {
        result.clear();
        return result;
    }

    let selector = Selector::parse("div.text-copy > strong").unwrap();
    let mut strong = vec![];

    for element in document.select(&selector) {
        if let Some(text) = element.text().next() {
            strong.push(text);
        }
    }

    let blur_text_selector = Selector::parse("div.blur-text").unwrap();
    for el in document.select(&blur_text_selector) {
        let date = el.text().collect::<String>();
        debug!("UPDATED ON: {}", date);
        if date.contains(r#"днес"#) {
            let today = Local::now().date_naive();
            result.insert(PUBLISHED_ON_KEY.to_owned(), today.to_string());
        } else if date.contains(r#"вчера"#) {
            let yesterday = Local::now().date_naive() - chrono::Duration::days(1);
            result.insert(PUBLISHED_ON_KEY.to_owned(), yesterday.to_string());
        } else {
            let published_on = match naive::NaiveDate::parse_from_str(date.trim(), "%d.%m.%y") {
                Ok(date) => date,
                Err(e) => {
                    info!("Error parsing date: {}", e);
                    continue;
                }
            };
            result.insert(PUBLISHED_ON_KEY.to_owned(), published_on.to_string());
        }
        break;
    }
    if strong.len() >= 2 {
        result.insert(YEAR_KEY.to_owned(), strong[0].to_owned());
        result.insert(LOCATION_KEY.to_owned(), strong[1].to_owned());
    } else if strong.len() == 1 {
        result.insert(YEAR_KEY.to_owned(), strong[0].to_owned());
    }
    get_vehicle_equipment(&document, &mut result);
    result
}

fn get_vehicle_equipment(document: &Html, data: &mut HashMap<String, String>) {
    let selector = Selector::parse("div.text-copy").unwrap();

    // Find all elements with the "description" class
    let mut equipment = vec![];
    for element in document.select(&selector) {
        // Extract the text content within the selected element
        let text = element.text().collect::<String>();
        // Split the text into lines
        let lines: Vec<&str> = text
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect();
        for l in lines {
            if l.contains(", ") {
                let values: Vec<String> = l.split(", ").map(|s| s.trim().to_owned()).collect();
                equipment.extend(values);
            }
        }
    }

    for eq in equipment.iter() {
        if eq.contains(r#"скорости"#) {
            data.insert(GEARBOX_KEY.to_owned(), eq.to_string());
        } else if eq.contains(r#"к.с."#) {
            let numeric = eq
                .chars()
                .filter(|&c| c.is_numeric())
                .collect::<String>()
                .parse::<i32>()
                .unwrap_or(0);
            data.insert(POWER_KEY.to_owned(), numeric.to_string());
        } else if eq.contains(r#"км"#) {
            let numeric = eq
                .chars()
                .filter(|&c| c.is_numeric())
                .collect::<String>()
                .parse::<i32>()
                .unwrap_or(0);
            data.insert(MILEAGE_KEY.to_owned(), numeric.to_string());
        } else if eq.contains("Газ/Бензин")
            || eq.contains("Дизел")
            || eq.contains("Бензин")
            || eq.contains("Електричество")
            || eq.contains("Хибрид")
        {
            data.insert(ENGINE_KEY.to_owned(), eq.to_string());
        }
    }
    let equipment_id = Equipment::get_equipment_as_u64(equipment);
    data.insert(EQUIPMENT_KEY.to_owned(), equipment_id.to_string());

    //println!("equipment * : {:?}", equipment);
}

#[cfg(test)]
mod test_cars_bg {
    use std::str::FromStr;

    use log::debug;
    use regex::Regex;

    use crate::{
        config::Equipment::{get_equipment_as_u64, get_values_by_equipment_id},
        model::enums::{Engine, Gearbox},
        utils::helpers::configure_log4rs,
        LOG_CONFIG,
    };

    use super::*;

    #[test]
    fn test_regex() {
        let input = r#"\n                                    \n                            17,990 лв.                    \n                                \n                    BMW 320 2.0D\n                \n                \n                                            Декември 2009,\n                                        Седан, Употребяван автомобил, В добро състояние, Дизел, 191 500км, Автоматични скорости, 177к.с., EURO 5, 2000см3, 4/5 врати, Сив            \n        "#;

        let _input1 = r#"Комби, Употребяван автомобил, В добро състояние, Газ/Бензин, 196 680км, Ръчни скорости, 201к.с., EURO 4, 2400см3, 4/5 врати, Черен"#;

        // Define regular expressions to match the desired patterns
        let fuel_type_regex = Regex::new(r"Дизел|Бензин|Газ/Бензин").unwrap();
        let mileage_regex = Regex::new(r"(\d+\s*\d*)км").unwrap();
        let power_regex = Regex::new(r"(\d+)\s*к\.с").unwrap();
        let gearbox_regex = Regex::new(r"Автоматични скорости|Ръчни скорости").unwrap();

        // Find and extract the desired information using regular expressions
        let fuel_type = fuel_type_regex
            .find(input)
            .map(|m| m.as_str())
            .unwrap_or("N/A");
        let mileage = mileage_regex
            .find(input)
            .map(|m| m.as_str().replace(' ', ""));
        let power = power_regex
            .find(input)
            .map(|m| m.as_str().replace("к.с", ""));

        let gearbox = gearbox_regex.find(input).map(|m| m.as_str());

        let engine = Engine::from_str(fuel_type).unwrap();
        let gear_box = Gearbox::from_str(gearbox.unwrap()).unwrap();
        // Print the extracted information
        debug!("Fuel Type: {:?}", engine);
        debug!("millage: {:?}", mileage);
        debug!("Power: {:?}", power);
        debug!("Gearbox: {:?}", gear_box);
    }

    #[test]
    fn test_eqipment() {
        let lines = ["Комфорт:", "Климатик", "Климатроник", "Ел.стъкла", 
            "Ел.огледала", "Стерео уредба", "Алуминиеви джанти", "DVD/TV", "Мултифункционален волан", "Сигурност:", "ABS, Airbag, Халогенни фарове, ASR/Тракшън контрол, Парктроник, Аларма, Имобилайзер, Центр. заключване, Застраховка, Старт-Стоп система", "Друго:", 
            "Автопилот", "Бордови компютър", "Навигационна система", "Теглич"];
        let result: Vec<String> = lines.iter().map(|&s| s.to_owned()).collect();
        let equipment_id = get_equipment_as_u64(result);
        debug!("equipment_id: {}", equipment_id);
        let values = get_values_by_equipment_id(equipment_id);
        debug!("values: {:?}", values);
    }

    #[tokio::test]
    async fn get_ids_test() {
        configure_log4rs(&LOG_CONFIG);
        let mut map = HashMap::new();
        //subm=1&add_search=1&typeoffer=1&priceFrom=18000&priceTo=30000&yearFrom=2007&yearTo=2011&page=32
        map.insert("subm".to_owned(), "1".to_owned());
        map.insert("add_search".to_owned(), "1".to_owned());
        map.insert("typeoffer".to_owned(), "1".to_owned());
        map.insert("priceFrom".to_owned(), "18000".to_owned());
        map.insert("priceTo".to_owned(), "30000".to_owned());
        map.insert("yearFrom".to_owned(), "2007".to_owned());
        map.insert("yearTo".to_owned(), "2011".to_owned());
        let records = search_cars_bg(map).await;
        debug!("records: {:?}", records.len());
        for record in records {
            sleep(Duration::from_millis(150));
            let _id = record.get("id").unwrap();
        }
    }

    #[tokio::test]
    async fn list_pages_test() {
        configure_log4rs(&LOG_CONFIG);
        let mut map = HashMap::new();
        map.insert("subm".to_owned(), "1".to_owned());
        map.insert("add_search".to_owned(), "1".to_owned());
        map.insert("typeoffer".to_owned(), "1".to_owned());
        map.insert("priceFrom".to_owned(), "18000".to_owned());
        map.insert("yearFrom".to_owned(), "2004".to_owned());
        let url = search_cars_bg_url(&map, 1);
        let listing = list_pages(&url).await;
        assert!(listing > 0);
        info!("Pages found: {}", listing);
    }

    #[tokio::test]
    async fn vehicles_per_pages_test() {
        configure_log4rs(&LOG_CONFIG);
        let mut map = HashMap::new();
        map.insert("subm".to_owned(), "1".to_owned());
        map.insert("add_search".to_owned(), "1".to_owned());
        map.insert("typeoffer".to_owned(), "1".to_owned());
        map.insert("priceFrom".to_owned(), "18000".to_owned());
        map.insert("yearFrom".to_owned(), "2004".to_owned());
        map.insert("page".to_owned(), "1".to_owned());
        let url = search_cars_bg_url(&map, 1);
        let listing = list_pages(&url).await;
        assert!(listing > 0);
        info!("Pages found: {}", listing);
        let html = get_pages_async(&url, false).await.unwrap();
        let result = read_listing(&html, false);
        assert!(result.len() > 0);
        assert_eq!(20, result.len());
    }

    #[tokio::test]
    async fn url_test() {
        let mut map = HashMap::new();
        //subm=1&add_search=1&typeoffer=1&priceFrom=18000&priceTo=30000&yearFrom=2007&yearTo=2011&page=32
        map.insert("subm".to_owned(), "1".to_owned());
        map.insert("add_search".to_owned(), "1".to_owned());
        map.insert("typeoffer".to_owned(), "1".to_owned());
        map.insert("priceFrom".to_owned(), "18000".to_owned());
        map.insert("priceTo".to_owned(), "30000".to_owned());
        map.insert("yearFrom".to_owned(), "2007".to_owned());
        map.insert("yearTo".to_owned(), "2011".to_owned());
        map.insert("page".to_owned(), "32".to_owned());
        let mut generated_url = r#"https://www.cars.bg/carslist.php?"#.to_owned();
        for (key, value) in map.iter() {
            generated_url = format!("{}{}={}&", generated_url, key, value);
        }
        info!("generated_url: {}", generated_url.trim_end_matches('&'));
        let url = search_cars_bg_url(&map, 1);
        info!("url: {}", url);
    }

    #[test]
    fn to_date() {
        configure_log4rs(&LOG_CONFIG);
        let date = naive::NaiveDate::parse_from_str("19.11.23", "%d.%m.%y").unwrap();
        info!("date: {}", date);
        assert_eq!("2023-11-19", date.to_string());
    }
}
