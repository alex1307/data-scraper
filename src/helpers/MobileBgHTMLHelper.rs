use crate::config::Equipment::get_equipment_as_u64;
use crate::helpers::CURRENCY_KEY;
use crate::helpers::DEALER_KEY;
use crate::helpers::ENGINE_KEY;
use crate::helpers::EQUIPMENT_KEY;
use crate::helpers::GEARBOX_KEY;
use crate::helpers::LOCATION_KEY;
use crate::helpers::MAKE_KEY;
use crate::helpers::MILEAGE_KEY;
use crate::helpers::MODEL_KEY;
use crate::helpers::PHONE_KEY;
use crate::helpers::POWER_KEY;
use crate::helpers::PRICE_KEY;
use crate::helpers::PUBLISHED_ON_KEY;
use crate::helpers::SOLD_KEY;
use crate::helpers::TOP_KEY;
use crate::helpers::VIEW_COUNT_KEY;
use crate::helpers::VIP_KEY;
use crate::helpers::YEAR_KEY;
use crate::model::enums::Currency;
use crate::model::enums::Engine;
use crate::model::enums::Gearbox;
use crate::model::records::MobileRecord;
use crate::model::VehicleDataModel::Resume;
use crate::utils::helpers::extract_ascii_latin;
use crate::utils::helpers::extract_date;
use crate::utils::helpers::extract_integers;
use crate::ENGINE_TXT;
use crate::GEARBOX_TXT;
use crate::POWER_TXT;
use crate::{BROWSER_USER_AGENT, MILLAGE_TXT, YEAR_TXT};

use encoding_rs::{UTF_8, WINDOWS_1251};
use log::info;
use log::{debug, error};

use regex::Regex;
use scraper::{ElementRef, Html, Selector};

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::ops::Index;

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

pub fn details2map(document: Html) -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("source".to_owned(), "mobile.bg".to_owned());
    let phone = if let Some(txt) = document.select(&PHONE_SELECTOR).next() {
        txt.text().collect::<Vec<_>>().join("")
    } else {
        "0000000000".to_string()
    };

    let address = if let Some(txt) = document.select(&ADDRESS_SELECTOR).next() {
        let location = txt.text().collect::<Vec<_>>().join("");
        location.split(',').collect::<Vec<_>>()[0].to_string()
    } else {
        "Unknown".to_string()
    };
    let is_dealer = document.select(&DEALER_SELECTOR).next().is_some();
    map.insert(PHONE_KEY.to_string(), phone);
    map.insert(DEALER_KEY.to_string(), (!is_dealer).to_string());
    map.insert(LOCATION_KEY.to_string(), address);

    if let Some(h1_element) = document.select(&DETAILS_HEADER_SELECTOR).next() {
        let text = h1_element.text().collect::<Vec<_>>().join(";");
        let values = text.split_whitespace().collect::<Vec<&str>>();

        for v in values.clone() {
            debug!("v: {}", v);
        }
        if values.len() < 2 {
            return HashMap::new();
        }
        map.insert(MAKE_KEY.to_string(), values[0].to_string());
        map.insert(MODEL_KEY.to_string(), values[1].to_string());
    }

    if document.select(&KAPARO_SELECTOR).count() > 0 {
        map.insert(SOLD_KEY.to_string(), "true".to_string());
    } else {
        map.insert(SOLD_KEY.to_string(), "false".to_string());
    }

    if let Some(element) = document.select(&UPDATED_ON_SELECTOR).next() {
        let txt = element.text().collect::<Vec<_>>().join(" ");
        if let Some(updated_on) = extract_date(&txt) {
            map.insert(PUBLISHED_ON_KEY.to_string(), updated_on);
        } else {
            map.insert(PUBLISHED_ON_KEY.to_string(), "".to_string());
        }
    }

    if document.select(&TOP_SELECTOR).count() > 0 {
        map.insert(TOP_KEY.to_string(), "true".to_string());
    } else if document.select(&VIP_SELECTOR).count() > 0 {
        map.insert(VIP_KEY.to_string(), "true".to_string());
    } else {
        map.insert(TOP_KEY.to_string(), "false".to_string());
        map.insert(VIP_KEY.to_string(), "false".to_string());
    }

    for element in document.select(&DILAR_SELECTOR) {
        let txt = element.text().collect::<Vec<_>>().join("_");
        let lines = txt.lines();

        for l in lines {
            if l.contains('_') {
                let v = l.split('_').collect::<Vec<&str>>();
                if v.len() >= 3 {
                    if ENGINE_TXT == v[1] {
                        map.insert(ENGINE_KEY.to_string(), v[2].to_string());
                    }
                    if GEARBOX_TXT == v[1] {
                        map.insert(GEARBOX_KEY.to_string(), v[2].to_string());
                    }

                    if v[1].contains(POWER_TXT) {
                        map.insert(POWER_KEY.to_string(), extract_integers(v[2])[0].to_string());
                    }

                    if v[1].contains(MILLAGE_TXT) {
                        if let Some(numeric_part) = v[2].split_whitespace().next() {
                            // Remove any non-numeric characters and convert to an integer
                            if let Ok(numeric_value) = numeric_part
                                .chars()
                                .filter(|&c| c.is_numeric())
                                .collect::<String>()
                                .parse::<i32>()
                            {
                                map.insert(MILEAGE_KEY.to_string(), numeric_value.to_string());
                            } else {
                                map.insert(MILEAGE_KEY.to_string(), "0".to_string());
                            }
                        } else {
                            error!("Milage not found for");
                        }
                    }

                    if v[1].contains(YEAR_TXT) {
                        debug!("v[2]: {}", v[2]);

                        // Remove any non-numeric characters and convert to an integer
                        if let Ok(numeric_value) = v[2]
                            .chars()
                            .filter(|&c| c.is_numeric())
                            .collect::<String>()
                            .parse::<i32>()
                        {
                            map.insert(YEAR_KEY.to_string(), numeric_value.to_string());
                        } else {
                            map.insert(YEAR_KEY.to_string(), "0".to_string());
                        }
                    }
                }
            }
        }
    }
    debug!("--> map: {:?}", map);
    for element in document.select(&ADV_ACT_SELECTOR) {
        let txt = element.text().collect::<Vec<_>>().join(" ");
        map.insert(
            VIEW_COUNT_KEY.to_string(),
            extract_integers(&txt)[0].to_string(),
        );
    }

    for element in document.select(&DETAILS_PRICE_SELECTOR) {
        let txt = element.text().collect::<Vec<_>>().join("");
        let (price, currency) = process_price(txt);
        map.insert(CURRENCY_KEY.to_string(), currency.to_string());
        map.insert(PRICE_KEY.to_string(), price.to_string());
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
            EQUIPMENT_KEY.to_string(),
            get_equipment_as_u64(extras).to_string(),
        );
    }
    map
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

pub fn make_and_mode(
    element: &ElementRef,
    models: HashMap<&str, Vec<&str>>,
) -> Option<(String, String)> {
    let selector = Selector::parse("td.valgtop a.mmm").unwrap();
    for e in element.select(&selector) {
        let inner_html = e.inner_html();
        let strings = inner_html.split_ascii_whitespace().collect::<Vec<&str>>();
        if strings.is_empty() || strings.len() < 2 {
            continue;
        }
        if models.is_empty() {
            return Some((strings[0].to_string(), strings[1].to_string()));
        }
    }

    None
}

pub fn is_top_or_vip(element: &ElementRef) -> bool {
    let top = vec!["top", "vip"];
    for value in top {
        let filter = format!(r#"img[alt="{}"][class="noborder"]"#, value);
        let selector = Selector::parse(&filter).unwrap();
        let img_element_exists = element.select(&selector).next().is_some();
        if img_element_exists {
            return true;
        }
    }

    false
}

pub fn is_sold(element: &ElementRef) -> bool {
    let filter = r#"img"#;
    let selector = Selector::parse(filter).unwrap();
    let images = element.select(&selector);

    for img_element in images {
        if let Some(src) = img_element.value().attr("src") {
            if src.contains("kaparirano.svg") {
                return true;
            }
        }
    }

    false
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
pub fn resume_info(
    html_content: &str,
    gearbox: Gearbox,
    engine: Engine,
    power: u32,
    dealer: bool,
) -> Vec<MobileRecord> {
    let document = Html::parse_document(&html_content);
    // Selector to find the price
    let price_selector = Selector::parse("span.price").unwrap();
    // Selector to find the description
    let rows_selector = Selector::parse("table.tablereset").unwrap();
    let make_model_selector = Selector::parse("td.valgtop > a.mmm").unwrap(); // Adjusted to be mo
    let a_selector = Selector::parse("a.mmm").unwrap();
    let logo_selector = Selector::parse("a.logoLink").unwrap();
    let href_regex = Regex::new(r"adv=(\d+)").unwrap(); // Extract make and model
    let mut resumes = vec![];
    let mut counter = 0;
    for element in document.select(&rows_selector) {
        let id: Option<String> = element.select(&a_selector).find_map(|element| {
            element.value().attr("href").and_then(|href| {
                // Extract the numeric value from the href attribute
                href_regex
                    .captures(href)
                    .and_then(|caps| caps.get(1).map(|match_| match_.as_str().to_string()))
            })
        });
        if id.is_none() {
            continue;
        }

        let mut resume = MobileRecord {
            id: id.unwrap(),
            source: "mobile.bg".to_string(),
            engine: engine.clone(),
            gearbox: gearbox.clone(),
            power,
            dealer,
            ..Default::default()
        };

        if counter > 20 {
            break;
        }

        if let Some(make_model_element) = element.select(&make_model_selector).next() {
            counter += 1;
            let d = make_model_element.text().collect::<Vec<_>>().join(" ");
            resume.title = d.clone();
            let make_model = d.split(" ").collect::<Vec<&str>>();
            if make_model.is_empty() || make_model.len() < 2 {
                continue;
            }

            if make_model.len() == 2 {
                resume.make = make_model[0].to_string();
                resume.model = make_model[1].to_string();
            } else if make_model.len() == 3 {
                resume.make = make_model[0].to_string();
                resume.model = make_model[1].to_string();
                resume.modification = make_model[2].to_string();
            } else {
                resume.make = make_model[0].to_string();
                resume.model = make_model[1].to_string();
                resume.modification = format!("{} {}", make_model[2], make_model[3]);
            }
        }

        if let Some(price_element) = element.select(&price_selector).next() {
            let price_text = price_element.text().collect::<Vec<_>>().join("");
            if price_text.contains("лв.") {
                resume.currency = Currency::BGN;
            } else if price_text.contains("EUR") {
                resume.currency = Currency::EUR;
            } else if price_text.contains("USD") {
                resume.currency = Currency::USD;
            }
            let mut price = price_text.replace(" лв.", "").replace(" ", ""); // Remove currency and spaces
            price = price.replace("EUR", "").replace(" ", ""); // Remove currency and spaces
            price = price.replace("USD", "").replace(" ", ""); // Remove currency and spaces
            resume.price = price.trim().parse::<u32>().unwrap_or(0);
        }

        let txt = element.inner_html();
        let start = txt.find(r#"дата на произв."#).unwrap();
        let substr = &txt[start..];
        let end = substr.find(r#"</td>"#).unwrap();
        let desc = &substr[..end];
        let (y, m) = extract_year_and_mileage(&desc);

        resume.year = y.parse::<u16>().unwrap_or(0);
        resume.mileage = m.parse::<u32>().unwrap_or(0);
        resume.location = extract_region(&desc).trim().to_string();

        if let Some(link) = element.select(&logo_selector).next() {
            info!("##### link: {:?}", link.attr("href").unwrap().to_string());
            resume.name = link.value().attr("href").unwrap().to_string();
        }

        resumes.push(resume.clone());
    }
    resumes
}

fn is_4_4(text: &str) -> bool {
    text.contains("4x4")
}

fn extract_region(text: &str) -> String {
    let region_regex = Regex::new(r"Регион: (.+?)\s").unwrap();
    region_regex
        .captures(text)
        .and_then(|cap| cap.get(1))
        .map(|match_| match_.as_str().to_string())
        .unwrap_or(String::default())
}

fn extract_year_and_mileage(text: &str) -> (String, String) {
    let year_regex = Regex::new(r"(\d{4}) г\.").unwrap();
    let mileage_regex = Regex::new(r"пробег - (\d+) км").unwrap();

    let year = year_regex
        .captures(text)
        .and_then(|cap| cap.get(1))
        .map(|match_| match_.as_str().to_string());

    let mileage = mileage_regex
        .captures(text)
        .and_then(|cap| cap.get(1))
        .map(|match_| match_.as_str().to_string());

    (
        year.unwrap_or("0".to_string()),
        mileage.unwrap_or("0".to_string()),
    )
}

#[cfg(test)]
mod test_listing {
    use encoding_rs::Encoding;
    use log::info;

    use crate::{utils::helpers::configure_log4rs, LOG_CONFIG};

    use super::*;

    #[test]
    fn test_get_pages() {
        configure_log4rs(&LOG_CONFIG);
        let file = std::fs::read("resources/test-data/mobile.bg/test.html").unwrap();

        //"windows-1251"
        let encoding = Encoding::for_label("windows-1251".as_bytes()).unwrap();
        let (decoded, _, _) = encoding.decode(&file);
        let utf8_html = UTF_8.encode(&decoded).0;
        let content = String::from_utf8_lossy(&utf8_html);
        let data = resume_info(
            content.to_string().as_str(),
            Gearbox::Manual,
            Engine::Petrol,
            0,
            false,
        );
        info!("data: {:?}", data);
    }
}
