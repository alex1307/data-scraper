use std::vec;

use scraper::{Html, Selector};

use super::mobile_bg::get_pages_async;
use lazy_static::lazy_static;

lazy_static! {
    static ref PRICE_SELECTOR: Selector = Selector::parse("div.offer-price > strong").unwrap();
    static ref VIEW_COUNT_SELECTOR: Selector = Selector::parse("div#offer_view_count").unwrap();
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

pub async fn read_details(url: &str) {
    let html = get_pages_async(url).await.unwrap();
    let document = Html::parse_document(&html);
    let price = document
        .select(&PRICE_SELECTOR)
        .next()
        .unwrap()
        .inner_html();
    let phone = document
        .select(&PHONE_SELECTOR)
        .next()
        .unwrap()
        .inner_html();
    println!("Price: {}", price);
    println!("Phone: {}", phone.replace("\n", "").trim());

    let selector = Selector::parse("div.description").unwrap();

    // Find all elements with the "description" class
    for element in document.select(&selector) {
        // Extract the text content within the selected element
        let text = element.text().collect::<String>();

        // Split the text into lines
        let lines: Vec<&str> = text
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect();
        println!("Description: {:?}", lines.last());
        if lines.contains(&"Изпрати запитване") {
            println!("Description: {:?}", lines.last());
        }
    }

    let selector = Selector::parse("div.text-copy").unwrap();

    // Find all elements with the "description" class
    for element in document.select(&selector) {
        // Extract the text content within the selected element
        let text = element.text().collect::<String>();

        // Split the text into lines
        let lines: Vec<&str> = text
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect();
        println!("text-copy: {:?}", lines.last());
    }

    let selector = Selector::parse("div.text-copy > strong").unwrap();
    let mut strong = vec![];

    // Find all elements with the "description" class
    for element in document.select(&selector) {
        // Extract the text content within the selected element
        if let Some(text) = element.text().next() {
            strong.push(text);
        }
    }

    println!("year: {:?}", strong.first().unwrap());
    println!("Location: {:?}", strong.last().unwrap());
}

#[cfg(test)]
mod test_cars_bg {
    use regex::Regex;

    use super::*;

    #[tokio::test]

    async fn test_read_details() {
        let url = "https://www.cars.bg/offer/652c45f2f4bef49e740767b4";
        read_details(url).await;
        // let view_count = get_view_counts("652c45f2f4bef49e740767b4");
        // println!("View count: {}", view_count.await.unwrap());
    }

    #[test]
    fn test_regex() {
        let input = "Джип, Употребяван автомобил, В добро състояние, Дизел, 178 000км, Автоматични скорости, 235к.с., EURO 4, 3000см3, 4/5 врати, Черен металик";
        let input1 = "Комби, Употребяван автомобил, В добро състояние, Газ/Бензин, 196 680км, Ръчни скорости, 201к.с., EURO 4, 2400см3, 4/5 врати, Черен";

        // Define regular expressions to match the desired patterns
        let fuel_type_regex = Regex::new(r"Дизел|Бензин|Газ/Бензин").unwrap();
        let mileage_regex = Regex::new(r"(\d+\s*\d*)км").unwrap();
        let power_regex = Regex::new(r"(\d+)\s*к\.с").unwrap();

        // Find and extract the desired information using regular expressions
        let fuel_type = fuel_type_regex
            .find(input1)
            .map(|m| m.as_str())
            .unwrap_or("N/A");
        let mileage = mileage_regex
            .find(input1)
            .map(|m| m.as_str().replace(" ", ""));
        let power = power_regex
            .find(input1)
            .map(|m| m.as_str().replace("к.с", ""));

        // Print the extracted information
        println!("Fuel Type: {}", fuel_type);
        println!("Mileage: {:?}", mileage);
        println!("Power: {:?}", power);
    }
}
