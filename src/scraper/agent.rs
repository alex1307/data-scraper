use crate::config::equipment::get_equipment_as_u64;
use crate::scraper::utils::extract_integers;
use crate::scraper::utils::get_milllage_and_year;
use crate::scraper::utils::is_sold;
use crate::scraper::utils::is_top_or_vip;
use crate::scraper::utils::make_and_mode;
use crate::model::details::MobileDetails;
use crate::model::enums::Payload;
use crate::model::enums::{Currency, Engine, Gearbox};
use crate::model::list::MobileList;
use crate::ACTION_DETAILS;
use crate::BROWSER_USER_AGENT;
use crate::DATE_FORMAT;
use crate::ENGINE_TXT;
use crate::GEARBOX_TXT;
use crate::NOT_FOUND_MSG;
use crate::POWER_TXT;

use encoding_rs::{UTF_8, WINDOWS_1251};
use log::error;
use log::info;

use scraper::{ElementRef, Html, Selector};

use super::form_data_request;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::str::FromStr;
lazy_static! {
    static ref TABLERESET_SELECTOR: Selector = Selector::parse("table.tablereset").unwrap();
    static ref PHONE_SELECTOR: Selector = Selector::parse("div.phone").unwrap();
    static ref DILAR_SELECTOR: Selector = Selector::parse("ul.dilarData").unwrap();
    static ref PRICE_SELECTOR: Selector = Selector::parse("span.price").unwrap();
    static ref TITLE_SELECTOR: Selector = Selector::parse("div.title").unwrap();
    static ref ADV_ACT_SELECTOR: Selector = Selector::parse("span.advact").unwrap();
    static ref DETAILS_PRICE_SELECTOR: Selector = Selector::parse("span#details_price").unwrap();
    static ref META_DESC_SELECTOR: Selector = Selector::parse("meta[name=description]").unwrap();
    static ref PAGE_NUMBERS_SELECTOR: Selector = Selector::parse("a.pageNumbers").unwrap();
    static ref TOP_MMM_SELECTOR: Selector = Selector::parse("td.valgtop a.mmm").unwrap();
    static ref DIV_MARGIN_SELECTOR: Selector =
        Selector::parse("div[style*=\"margin-bottom:5px;\"]").unwrap();
}

pub async fn scrape(url: &str) -> Payload<HashMap<String, String>> {
    if url.contains(ACTION_DETAILS) {
        let m = details2map(url).await;
        if let Some(_v) = m.get("error") {
            Payload::Error(m)
        } else {
            Payload::Value(m)
        }
    } else {
        Payload::Data(list2map(url).await)
    }
}

pub async fn process_link(url: &str) -> Vec<HashMap<String, String>> {
    if url.contains(ACTION_DETAILS) {
        vec![details2map(url).await]
    } else {
        list2map(url).await
    }
}

async fn details2map(url: &str) -> HashMap<String, String> {
    info!("Processing details {}", url);

    let mut map = HashMap::new();
    let html = match get_pages_async(url).await {
        Ok(v) => v,
        Err(e) => {
            error!("Error getting details {}", e);
            return map;
        }
    };
    if let Some(adv_value) = get_id_from_url(url.to_string()) {
        map.insert("id".to_string(), adv_value);
        if html.contains(NOT_FOUND_MSG) {
            map.insert("error".to_string(), "Not found".to_string());
            return map;
        }
    } else {
        return HashMap::new();
    }

    let document = Html::parse_document(&html);

    map.insert("type".to_string(), "DETAILS".to_string());

    let phone = if let Some(txt) = document.select(&PHONE_SELECTOR).next() {
        txt.text().collect::<Vec<_>>().join("")
    } else {
        "0000000000".to_string()
    };

    map.insert("phone".to_string(), phone);

    for element in document.select(&DILAR_SELECTOR) {
        let txt = element.text().collect::<Vec<_>>().join("_");
        let lines = txt.lines();
        for l in lines {
            if l.contains('_') {
                let v = l.split('_').collect::<Vec<&str>>();
                if v.len() >= 3 {
                    if ENGINE_TXT == v[1] {
                        map.insert("engine".to_string(), v[2].to_string());
                    }
                    if GEARBOX_TXT == v[1] {
                        map.insert("gearbox".to_string(), v[2].to_string());
                    }

                    if v[1].contains(POWER_TXT) {
                        map.insert("power".to_string(), extract_integers(v[2])[0].to_string());
                    }
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    for element in document.select(&ADV_ACT_SELECTOR) {
        let txt = element.text().collect::<Vec<_>>().join(" ");
        map.insert(
            "view_count".to_string(),
            extract_integers(&txt)[0].to_string(),
        );
    }

    for element in document.select(&DETAILS_PRICE_SELECTOR) {
        let txt = element.text().collect::<Vec<_>>().join("");
        let (price, currency) = process_price(txt);
        map.insert("currency".to_string(), currency.to_string());
        map.insert("price".to_string(), price.to_string());
    }

    let divs = document.select(&DIV_MARGIN_SELECTOR);
    let mut extras = vec![];
    for div in divs {
        extras.push(
            div.text()
                .collect::<String>()
                .replace('•', "")
                .trim()
                .to_string(),
        );
    }
    if !&extras.is_empty() {
        map.insert(
            "equipment".to_string(),
            get_equipment_as_u64(extras).to_string(),
        );
    }
    map
}

async fn list2map(url: &str) -> Vec<HashMap<String, String>> {
    let html = get_pages_async(url).await.unwrap();
    let created_on = chrono::Utc::now().format(DATE_FORMAT).to_string();
    let document = Html::parse_document(&html);

    let mut results = vec![];
    for element in document.select(&TABLERESET_SELECTOR) {
        let mut result = HashMap::new();
        result.insert("type".to_string(), "LIST".to_string());
        let prices = extract_price(&element);
        let make_and_mode = make_and_mode(&element, HashMap::new());
        if let Some(url) = get_url(&element) {
            let id = get_id_from_url(url.clone());
            if id.is_some() && prices.is_some() && make_and_mode.is_some() {
                result.insert("id".to_string(), id.unwrap().to_string());
                let (make, model) = make_and_mode.unwrap();
                result.insert("make".to_string(), make);
                result.insert("model".to_string(), model);
                let (price, currency) = prices.unwrap();
                result.insert("price".to_string(), price.to_string());
                result.insert("currency".to_string(), currency.to_string());
                result.insert("created_on".to_string(), created_on.clone());
                let is_promoted = is_top_or_vip(&element);
                result.insert("promoted".to_string(), is_promoted.to_string());
                result.insert("sold".to_string(), is_sold(&element).to_string());
                let (year, millage) = get_milllage_and_year(&element, is_promoted);
                if (0, 0) == (year, millage) {
                    error!("Failed to get year and millage for {}", url);
                }
                result.insert("year".to_string(), year.to_string());
                result.insert("millage".to_string(), millage.to_string());
                results.push(result);
            }
        }
    }
    results
}

pub fn parse_details(url: &str) -> Result<MobileDetails, Box<dyn std::error::Error>> {
    let html = get_pages(url)?;
    if html.contains(NOT_FOUND_MSG) {
        return Err("not found".into());
    }
    let document = Html::parse_document(&html);
    let phone = document
        .select(&PHONE_SELECTOR)
        .next()
        .unwrap()
        .text()
        .collect::<Vec<_>>()
        .join("");
    let adv_value = url
        .split('&')
        .find(|s| s.starts_with("adv="))
        .unwrap()
        .split('=')
        .last()
        .unwrap();
    info!("Phone: {}", phone);
    let mut details = MobileDetails::new(adv_value.to_string(), phone);
    for element in document.select(&DILAR_SELECTOR) {
        let txt = element.text().collect::<Vec<_>>().join("_");
        let lines = txt.lines();
        for l in lines {
            if l.contains('_') {
                let v = l.split('_').collect::<Vec<&str>>();
                if v.len() >= 3 {
                    if "Тип двигател" == v[1] {
                        details.engine = Engine::from_str(v[2])?;
                    }
                    if "Скоростна кутия" == v[1] {
                        details.gearbox = Gearbox::from_str(v[2])?;
                    }

                    if v[1].contains("Мощност") {
                        let power = extract_integers(v[2]);
                        details.power = power[0] as u16;
                    }
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
    for element in document.select(&ADV_ACT_SELECTOR) {
        let txt = element.text().collect::<Vec<_>>().join(" ");
        details.view_count = extract_integers(&txt)[0];
    }
    for element in document.select(&DETAILS_PRICE_SELECTOR) {
        let txt = element.text().collect::<Vec<_>>().join("");
        let (price, currency) = process_price(txt);
        details.currency = currency;
        details.price = price;
    }

    let divs = document.select(&DIV_MARGIN_SELECTOR);
    let mut extras = vec![];
    for div in divs {
        extras.push(div.text().collect::<String>().replace('•', ""));
    }
    details.extras = extras.clone();
    if !&extras.is_empty() {
        details.equipment = get_equipment_as_u64(extras);
    }
    Ok(details)
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

pub fn get_links(html: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let document = Html::parse_document(html);
    let mut links = vec![];
    for element in document.select(&PAGE_NUMBERS_SELECTOR) {
        let txt = element.value().attr("href").unwrap_or("");
        if links.contains(&txt.to_string()) {
            continue;
        } else {
            links.push(txt.to_string());
        }
    }
    Ok(links)
}

fn extract_price(element: &ElementRef) -> Option<(u32, Currency)> {
    let element = match element.select(&PRICE_SELECTOR).next() {
        Some(e) => e,
        None => return None, // return None if no <a> element is found
    };
    let price_element = element.text().collect::<Vec<_>>().join("");
    Some(process_price(price_element))
}

fn process_price(text: String) -> (u32, Currency) {
    let contains_numeric = text.chars().any(|c| c.is_numeric());
    if !contains_numeric {
        return (0, Currency::BGN);
    }
    let v = text.replace(' ', "");
    let v1 = v.replace("&nbsp;", "");
    let (price_str, currency) = if v1.contains("USD") {
        (v1.trim_end_matches("USD"), Currency::USD)
    } else if v1.contains("EUR") {
        (v1.trim_end_matches("EUR"), Currency::EUR)
    } else {
        (v1.trim_end_matches("лв."), Currency::BGN)
    };
    let price = match price_str.parse::<f32>() {
        Ok(p) => p.floor() as u32,
        Err(_) => return (0, Currency::BGN), // return None if the string cannot be parsed as u32
    };
    (price, currency)
}

fn get_url(element: &ElementRef) -> Option<String> {
    match element.select(&TOP_MMM_SELECTOR).next() {
        Some(e) => {
            let href = e.value().attr("href").unwrap();
            Some(href.to_owned())
        }
        None => None,
    }
}

fn get_id_from_url(url: String) -> Option<String> {
    let id = url
        .split('&')
        .find(|s| s.starts_with("adv="))?
        .split('=')
        .last()?;
    Some(id.to_owned())
}

pub fn search_form_data(
    input: &form_data_request::Request,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(BROWSER_USER_AGENT)
        .build()?;
    let body: Vec<u8> = client
        .post("https://www.mobile.bg/pcgi/mobile.cgi")
        .form(&input.to_form_data())
        .send()?
        .bytes()
        .unwrap()
        .to_vec();
    let (html, _, _) = WINDOWS_1251.decode(&body);
    let utf8_html = UTF_8.encode(&html).0;
    let response = String::from_utf8_lossy(&utf8_html);
    Ok(response.to_string())
}

pub async fn get_pages_async(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .user_agent(BROWSER_USER_AGENT)
        .build()?;
    let https_url = format!("https:{}", url);
    let body: Vec<u8> = client.get(&https_url).send().await?.bytes().await?.to_vec();
    // Decode the byte array using the Windows-1251 encoding
    let (html, _, _) = WINDOWS_1251.decode(&body);
    // Convert the decoded text to UTF-8
    let utf8_html = UTF_8.encode(&html).0;
    let response = String::from_utf8_lossy(&utf8_html);
    Ok(response.to_string())
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

pub fn get_vehicles_prices(html: &str) -> Vec<MobileList> {
    let created_on = chrono::Utc::now().format(DATE_FORMAT).to_string();
    let document = Html::parse_document(html);

    let mut vehicle_prices = vec![];
    for element in document.select(&TABLERESET_SELECTOR) {
        let prices = extract_price(&element);
        let make_and_mode = make_and_mode(&element, HashMap::new());
        if let Some(url) = get_url(&element) {
            let id = get_id_from_url(url.clone());
            match id.is_some() && prices.is_some() && make_and_mode.is_some() {
                true => {
                    let (make, model) = make_and_mode.unwrap();
                    let (price, currency) = prices.unwrap();
                    let mut vehicle_price = MobileList::new(
                        id.unwrap(),
                        make,
                        model,
                        price,
                        currency,
                        created_on.clone(),
                    );
                    vehicle_price.promoted = is_top_or_vip(&element);
                    vehicle_price.sold = is_sold(&element);
                    let (year, millage) = get_milllage_and_year(&element, vehicle_price.promoted);
                    vehicle_price.year = year as u16;
                    vehicle_price.millage = millage;
                    vehicle_prices.push(vehicle_price);
                }
                false => (),
            }
        }
    }
    info!("Found {} vehicles", vehicle_prices.len());
    vehicle_prices
}
