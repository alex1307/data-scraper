use crate::model::enums::Currency;
use crate::model::enums::Engine;
use crate::model::enums::Gearbox;
use crate::model::VehicleRecord::MobileRecord;

use crate::utils::helpers::extract_ascii_latin;
use crate::utils::helpers::extract_make;
use crate::BROWSER_USER_AGENT;

use encoding_rs::{UTF_8, WINDOWS_1251};

use log::debug;

use regex::Regex;
use scraper::{ElementRef, Html, Selector};

use lazy_static::lazy_static;
use std::str::FromStr;

lazy_static! {
    static ref TABLERESET_SELECTOR: Selector = Selector::parse("table.tablereset").unwrap();
    static ref DETAILS_HEADER_SELECTOR: Selector = Selector::parse("h1").unwrap();
    static ref KAPARO_SELECTOR: Selector = Selector::parse("div.kaparo").unwrap();
    static ref TOP_SELECTOR: Selector = Selector::parse("td.img.TOP").unwrap();
    static ref VIP_SELECTOR: Selector = Selector::parse("td.img.VIP").unwrap();
    static ref PHONE_SELECTOR: Selector = Selector::parse("div.phone").unwrap();
    static ref DEALER_SELECTOR: Selector = Selector::parse("div.AG > strong").unwrap();
    static ref ADDRESS_SELECTOR: Selector = Selector::parse("div.adress").unwrap();
    static ref DILAR_SELECTOR: Selector = Selector::parse("ul.dilarData").unwrap();
    static ref PRICE_SELECTOR: Selector = Selector::parse("span.price").unwrap();
    static ref TITLE_SELECTOR: Selector = Selector::parse("div.title").unwrap();
    static ref ADV_ACT_SELECTOR: Selector = Selector::parse("span.advact").unwrap();
    static ref UPDATED_ON_SELECTOR: Selector =
        Selector::parse("span[style=\"color:#999999\"]").unwrap();
    static ref DETAILS_PRICE_SELECTOR: Selector = Selector::parse("span#details_price").unwrap();
    static ref META_DESC_SELECTOR: Selector = Selector::parse("meta[name=description]").unwrap();
    static ref PAGE_NUMBERS_SELECTOR: Selector = Selector::parse("a.pageNumbers").unwrap();
    static ref TOP_MMM_SELECTOR: Selector = Selector::parse("td.valgtop a.mmm").unwrap();
    static ref INPUT_TYPE_HIDDEN: Selector = Selector::parse("input[name=slink]").unwrap();
    static ref DIV_MARGIN_SELECTOR: Selector =
        Selector::parse("div[style*=\"margin-bottom:5px;\"]").unwrap();
}

pub fn get_header_data(html: &str) -> Result<String, Box<dyn std::error::Error>> {
    let fragment = Html::parse_document(html);
    let description = fragment
        .select(&META_DESC_SELECTOR)
        .next()
        .unwrap()
        .value()
        .attr("content")
        .unwrap()
        .to_string();
    Ok(description)
}

pub fn get_metadata_links(html: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let document = Html::parse_document(html);
    let mut links = vec![];
    for element in document.select(&PAGE_NUMBERS_SELECTOR) {
        let txt = element.value().attr("href").unwrap_or("");
        if links.contains(&txt.to_string()) {
            continue;
        }
        links.push(txt.to_string());
    }
    Ok(links)
}

pub fn get_url(element: &ElementRef) -> Option<String> {
    match element.select(&TOP_MMM_SELECTOR).next() {
        Some(e) => {
            let href = e.value().attr("href").unwrap();
            Some(href.to_owned())
        }
        None => None,
    }
}

pub async fn get_pages_async(
    url: &str,
    encoding: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .user_agent(BROWSER_USER_AGENT)
        .build()?;
    let body: Vec<u8> = client.get(url).send().await?.bytes().await?.to_vec();
    debug!("body: {}", body.len());
    if encoding {
        let (html, _, _) = WINDOWS_1251.decode(&body);
        // Convert the decoded text to UTF-8
        let utf8_html = UTF_8.encode(&html).0;
        let response = String::from_utf8_lossy(&utf8_html);
        Ok(response.to_string())
    } else {
        let response = String::from_utf8_lossy(&body);
        debug!("response: {}", response.len());
        Ok(response.to_string())
    }
}

pub fn get_pages(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(BROWSER_USER_AGENT)
        .build()?;
    let https_url = format!("https:{}", url);
    let body: Vec<u8> = client.get(https_url).send()?.bytes().unwrap().to_vec();
    // Decode the byte array using the Windows-1251 encoding
    let (html, _, _) = WINDOWS_1251.decode(&body);
    // Convert the decoded text to UTF-8
    let utf8_html = UTF_8.encode(&html).0;
    let response = String::from_utf8_lossy(&utf8_html);
    Ok(response.to_string())
}

pub fn slink(html: &str) -> String {
    let document = Html::parse_document(html);
    let mut result = "".to_string();

    for element in document.select(&INPUT_TYPE_HIDDEN) {
        if let Some(txt) = element.value().attr("value") {
            result = txt.to_string();
            break; // Exit the loop once a value is found
        }
    }

    result
}

pub fn get_milllage_and_year(element: &ElementRef, is_promoted: bool) -> (u32, u32) {
    let filter = match is_promoted {
        true => r#"td[colspan="3"]"#,
        false => r#"td[colspan="4"]"#,
    };

    let selector = Selector::parse(filter).unwrap();
    let mut txt = element.select(&selector).next().unwrap().inner_html();
    txt = extract_ascii_latin(&txt);

    extract_numbers(&txt)
}

pub fn extract_numbers(input: &str) -> (u32, u32) {
    if input.is_empty() {
        return (0, 0);
    }

    let contains_numeric = input.chars().any(|c| c.is_numeric());
    if !contains_numeric {
        return (0, 0);
    }
    let re = Regex::new(r"\d+").unwrap();
    let mut numbers: Vec<u32> = Vec::new();
    for mat in re.find_iter(input) {
        match mat.as_str().parse() {
            Ok(n) => numbers.push(n),
            Err(_) => {
                // Handle invalid number here.
                println!("Invalid number: {}", mat.as_str());
                return (0, 0);
            }
        }
    }

    if numbers.len() < 2 {
        return (0, 0);
    }
    let n = numbers[0];
    let k = numbers[1];

    (n, k)
}
pub fn get_vehicles(html_content: &str) -> Vec<MobileRecord> {
    let document = Html::parse_document(html_content);
    // Selector to find the price
    let price_selector = Selector::parse("div.price div").unwrap();
    // Selector to find the description
    let rows_selector = Selector::parse("div.item").unwrap();
    let make_model_selector = Selector::parse("div.zaglavie a.title").unwrap(); // Adjusted to be mo
    let params_selector = Selector::parse("div.params span").unwrap();
    let seller_name = Selector::parse("div.sInfo div.name a").unwrap();
    let location = Selector::parse("div.sInfo div.location").unwrap();
    let mut vehicles = vec![];

    for element in document.select(&rows_selector) {
        let mut vehicle = MobileRecord {
            id: "".to_string(),
            dealer: true,
            ..Default::default()
        };
        if let Some(make_model_element) = element.select(&make_model_selector).next() {
            if let Some(href) = make_model_element.value().attr("href") {
                // Extract the ID from the URL
                let parts: Vec<&str> = href.split('-').collect();
                if let Some(id_part) = parts.get(1) {
                    let id = id_part.split('/').next().unwrap_or("");
                    vehicle.id = id.to_string();
                } else {
                    continue;
                }
            }
            let title_txt = make_model_element
                .text()
                .collect::<String>()
                .trim()
                .to_string();
            let mut make_model = title_txt.split_whitespace().collect::<Vec<_>>();
            if let Some(last) = make_model.last() {
                if last.ends_with("...")
                    || regex::Regex::new(r"[\u0400-\u04FF\u0500-\u052F\u2DE0-\u2DFF\uA640-\uA69F]")
                        .unwrap()
                        .is_match(last)
                {
                    make_model.pop();
                }
            }

            let (make, model, title) = extract_make(make_model);
            vehicle.make = make;
            vehicle.model = model;
            vehicle.title = title;
        }

        if let Some(price_element) = element.select(&price_selector).next() {
            let inner = price_element.inner_html();
            let price = inner.chars().filter(|c| c.is_numeric()).collect::<String>();
            vehicle.price = price.parse::<u32>().unwrap_or(0);
            if inner.contains("USD") {
                vehicle.currency = Currency::BGN;
            } else if inner.contains("EUR") {
                vehicle.currency = Currency::EUR;
            } else {
                vehicle.currency = Currency::BGN;
            }
        }

        for param in element.select(&params_selector) {
            let txt = param.text().collect::<String>().trim().to_string();
            if txt.contains("г.") {
                let year = txt.chars().filter(|c| c.is_numeric()).collect::<String>();
                vehicle.year = year.parse::<u16>().unwrap_or(0);
                continue;
            }

            if txt.contains("км") {
                let mileage = txt.chars().filter(|c| c.is_numeric()).collect::<String>();
                vehicle.mileage = mileage.parse::<u32>().unwrap_or(0);
                continue;
            }

            if txt.contains("к.с.") {
                let power = txt.chars().filter(|c| c.is_numeric()).collect::<String>();
                vehicle.power = power.parse::<u32>().unwrap_or(0);
                continue;
            }

            if txt.contains("куб.см") {
                let cc = txt.chars().filter(|c| c.is_numeric()).collect::<String>();
                vehicle.cc = cc.parse::<u32>().unwrap_or(0);
                continue;
            }
            if let Ok(engine) = Engine::from_str(&txt) {
                vehicle.engine = engine.clone();
                continue;
            }

            if let Ok(gearbox) = Gearbox::from_str(&txt) {
                vehicle.gearbox = gearbox;
                continue;
            }
        }

        if let Some(seller_name_element) = element.select(&seller_name).next() {
            let url = seller_name_element.value().attr("href").unwrap_or("");
            vehicle.dealer_url = url.to_string();
            vehicle.name = seller_name_element
                .text()
                .collect::<String>()
                .trim()
                .to_string();
        }

        if let Some(location_element) = element.select(&location).next() {
            vehicle.location = location_element
                .text()
                .collect::<String>()
                .trim()
                .to_string();
        }

        vehicles.push(vehicle);
    }
    vehicles
}

#[cfg(test)]
mod test_listing {

    use std::io::Read;

    use log::info;

    use crate::{utils::helpers::configure_log4rs, LOG_CONFIG};

    use super::*;

    #[test]
    fn test_get_pages() {
        configure_log4rs(&LOG_CONFIG);
        let mut file = std::fs::File::open("resources/test-data/mobile.bg/test.html").unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let data = get_vehicles(&contents.to_string());
        info!("data: {:?}", data);
    }
}
