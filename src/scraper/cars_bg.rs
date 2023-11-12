use std::collections::HashMap;

use scraper::{Html, Selector};

use crate::config::equipment;

use super::mobile_bg::get_pages_async;
use lazy_static::lazy_static;

lazy_static! {
    static ref PRICE_SELECTOR: Selector = Selector::parse("div.offer-price > strong").unwrap();
    static ref VIEW_COUNT_SELECTOR: Selector = Selector::parse("div#offer_view_count").unwrap();
    static ref MAKE_MODEL_SELECTOR: Selector = Selector::parse("div.text-copy h2").unwrap();
    static ref PHONE_SELECTOR: Selector = Selector::parse("a.a_call_link > div").unwrap();
}

pub async fn get_view_counts(id: &str) -> Result<String, reqwest::Error> {
    let url = format!("https://stats.cars.bg/add/?object_id={}", id);
    reqwest::get(&url).await.unwrap();
    let url = format!("https://stats.cars.bg/get/?object_id={}", id);
    let response = reqwest::get(&url).await.unwrap();
    let text = response.text().await.unwrap();
    Ok(text)
}

pub async fn get_ids(url: &str) -> Result<Vec<String>, reqwest::Error> {
    let html = get_pages_async(url, false).await.unwrap();
    let document = Html::parse_document(&html);
    let selector = Selector::parse("div.offer-item").unwrap();
    let mut ids = vec![];
    for element in document.select(&selector) {
        let id = element.value().attr("data-id").unwrap();
        ids.push(id.to_owned());
    }
    Ok(ids)
}

pub async fn read_details(id: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    let url = format!("https://www.cars.bg/offer/{}", id);
    let html = get_pages_async(&url, false).await.unwrap();
    if html.contains(r#"Частно лице"#) {
        result.insert("dealer".to_owned(), "false".to_owned());
    } else {
        result.insert("dealer".to_owned(), "true".to_owned());
    }
    let document = Html::parse_document(&html);
    let price = document
        .select(&PRICE_SELECTOR)
        .next()
        .unwrap()
        .inner_html();
    let numeric = price.chars()
           .filter(|&c| c.is_numeric())
           .collect::<String>()
           .parse::<i32>().unwrap_or(0);
    result.insert("price".to_owned(), numeric.to_string());

    let phone = document
        .select(&PHONE_SELECTOR)
        .next()
        .unwrap()
        .inner_html();
    result.insert("phone".to_owned(), phone.replace("\n", "").trim().to_string());
    let make_model = document
        .select(&MAKE_MODEL_SELECTOR)
        .next()
        .unwrap()
        .inner_html().split_ascii_whitespace()
        .map(|s| s.to_owned())
        .collect::<Vec<String>>();
    if make_model.len() > 1 {
        result.insert("make".to_owned(), make_model[0].to_owned());
        result.insert("model".to_owned(), make_model[1].to_owned());
    } else {
        result.clear();
        return result; 
    }
    let view_count = get_view_counts(id).await.unwrap();
    result.insert("view_count".to_owned(), view_count);
    
    let selector = Selector::parse("div.text-copy > strong").unwrap();
    let mut strong = vec![];

    for element in document.select(&selector) {
       if let Some(text) = element.text().next() {
            strong.push(text);
        }
    }
    
    if strong.len() >= 2 {
        result.insert("year".to_owned(), strong[0].to_owned());
        result.insert("location".to_owned(), strong[1].to_owned());
    }
    get_vehicle_equipment(&document, &mut result);
    result.insert("id".to_owned(), id.to_owned());
    return result;
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
        for l in lines{
            if l.contains(", ") {
                let values:Vec<String> = l.split(", ")
                    .map(|s| s.trim().to_owned())
                    .collect();
                equipment.extend(values);
            }
        }
    }

    let engine = equipment[4].to_owned();
    data.insert("engine".to_owned(), engine);
    let mileage = equipment[5].to_owned();
    let numeric = mileage.chars()
           .filter(|&c| c.is_numeric())
           .collect::<String>()
           .parse::<i32>().unwrap_or(0);
    data.insert("mileage".to_owned(), numeric.to_string());
    let gearbox = equipment[6].to_owned();
    data.insert("gearbox".to_owned(), gearbox);
    let power = equipment[7].to_owned();
    let numerice = power.chars()
           .filter(|&c| c.is_numeric())
           .collect::<String>()
           .parse::<i32>().unwrap_or(0);
    data.insert("power".to_owned(), numerice.to_string());
    let equipment_id = equipment::get_equipment_as_u64(equipment, &equipment::CARS_BG_EQUIPMENT);
    data.insert("equipment".to_owned(), equipment_id.to_string());
    
    //println!("equipment * : {:?}", equipment);

}

#[cfg(test)]
mod test_cars_bg {
    use std::str::FromStr;

    use regex::Regex;
    use serde_json::from_str;

    use crate::{
        config::equipment::{get_equipment_as_u64, get_values_by_equipment_id, CARS_BG_EQUIPMENT},
        model::enums::{Engine, Gearbox},
    };

    use super::*;

    #[tokio::test]

    async fn test_read_details() {
        
        let map = read_details("646c6bca7a84bc1a4e050592").await;
        println!("map: {:?}", map); 
        // let view_count = get_view_counts("63da044e6cdd8996410c5143");
        // println!("View count: {}", view_count.await.unwrap());
    }

    #[test]
    fn test_regex() {
        let input = r#"\n                                    \n                            17,990 лв.                    \n                                \n                    BMW 320 2.0D\n                \n                \n                                            Декември 2009,\n                                        Седан, Употребяван автомобил, В добро състояние, Дизел, 191 500км, Автоматични скорости, 177к.с., EURO 5, 2000см3, 4/5 врати, Сив            \n        "#;


        let input1 = r#"Комби, Употребяван автомобил, В добро състояние, Газ/Бензин, 196 680км, Ръчни скорости, 201к.с., EURO 4, 2400см3, 4/5 врати, Черен"#;

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
            .map(|m| m.as_str().replace(" ", ""));
        let power = power_regex
            .find(input)
            .map(|m| m.as_str().replace("к.с", ""));

        let gearbox = gearbox_regex.find(input).map(|m| m.as_str());

        let engine = Engine::from_str(fuel_type).unwrap();
        let gear_box = Gearbox::from_str(gearbox.unwrap()).unwrap();
        // Print the extracted information
        println!("Fuel Type: {:?}", engine);
        println!("Mileage: {:?}", mileage);
        println!("Power: {:?}", power);
        println!("Gearbox: {:?}", gear_box);
    }

    #[test]
    fn test_eqipment() {
        let lines = ["Комфорт:", "Климатик", "Климатроник", "Ел.стъкла", 
            "Ел.огледала", "Стерео уредба", "Алуминиеви джанти", "DVD/TV", "Мултифункционален волан", "Сигурност:", "ABS, Airbag, Халогенни фарове, ASR/Тракшън контрол, Парктроник, Аларма, Имобилайзер, Центр. заключване, Застраховка, Старт-Стоп система", "Друго:", 
            "Автопилот", "Бордови компютър", "Навигационна система", "Теглич"];
        let result: Vec<String> = lines.iter().map(|&s| s.to_owned()).collect();
        println!("result: {:?}", &CARS_BG_EQUIPMENT.len());
        let equipment_id = get_equipment_as_u64(result, &CARS_BG_EQUIPMENT);
        println!("equipment_id: {}", equipment_id);
        for (key, value) in CARS_BG_EQUIPMENT.iter() {
            let mask = 2_u64.pow(*key as u32);
            if equipment_id & mask == mask {
                println!("value: {}", value);
            }
        }
        let values = get_values_by_equipment_id(equipment_id, &CARS_BG_EQUIPMENT);
        println!("values: {:?}", values);
    }

    #[tokio::test]
    async fn get_ids_test() {
        let url = r#"https://www.cars.bg/carslist.php?subm=1&add_search=1&typeoffer=1&priceFrom=18000&priceTo=30000&yearFrom=2007&yearTo=2011&page=32"#;
        let html = get_pages_async(url, false).await.unwrap();
        let document = Html::parse_document(&html);
        let selector = Selector::parse("div.mdc-card__primary-action").unwrap();
        let mut fragments_counter = 1;
        for element in document.select(&selector) {
            let html_fragment = Html::parse_fragment(element.inner_html().as_str());
            let selector = Selector::parse("a").unwrap();
            for e in html_fragment.select(&selector) {
                println!("{}: {:?}", fragments_counter, e.value().attr("href").unwrap());
            }

            let h6_selector = Selector::parse("h6").unwrap();
            for v in html_fragment.select(&h6_selector){
               println!("--> : {:?}",v.inner_html().split_ascii_whitespace()
                    .map(|s| s.to_owned())
                    .collect::<Vec<String>>());
            }

            let card_2nd_line_selector = Selector::parse("div.card__secondary").unwrap();   
            for v in html_fragment.select(&card_2nd_line_selector){
               println!("--> : {:?}",v.inner_html().split_ascii_whitespace()
                    .map(|s| s.to_owned())
                    .collect::<Vec<String>>());
            }
            let card_footer_selector = Selector::parse("div.card__footer").unwrap();   
            for v in html_fragment.select(&card_footer_selector){
               println!("--> : {:?}",v.inner_html().split_ascii_whitespace()
                    .map(|s| s.to_owned())
                    .collect::<Vec<String>>());
            }
            fragments_counter += 1;
        }

        let  total_number_selector = Selector::parse("span.milestoneNumberTotal").unwrap();
        let element = document.select(&total_number_selector).next().unwrap();
        let total_number = element.inner_html();
        println!("totalNumber: {}", total_number.replace("\n", "").trim());
    }

}
