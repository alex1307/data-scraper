pub mod currency;
pub mod data_processor;
pub mod mobile_utils;
pub mod model;
pub mod utils;

use encoding_rs::{UTF_8, WINDOWS_1251};
use log::{info, debug};
use reqwest::blocking::Client;
use scraper::{ElementRef, Html, Selector};
use std::collections::HashMap;

use crate::mobile_scraper::mobile_utils::{extract_ascii_latin, extract_numbers};

use self::{
    currency::Currency,
    model::{SearchRequest, VehiclePrice},
};
pub const SEARCH_URL: &str = "https://www.mobile.bg/pcgi/mobile.cgi";
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut form_data = HashMap::new();
    form_data.insert("topmenu", "1".to_owned());
    form_data.insert("rub_pub_save", 1.to_string());
    form_data.insert("act", 3.to_string());
    form_data.insert("f0", "151.251.224.1".to_owned());
    form_data.insert("f5", "Mercedes-Benz".to_owned());
    form_data.insert("f6", "C".to_owned());
    form_data.insert("f10", 2010.to_string());
    form_data.insert("f11", 2012.to_string());
    form_data.insert("f20", 1.to_string());

    let res = client
        .get("https://www.mobile.bg/pcgi/mobile.cgi?act=4&adv=11661885115343676&slink=ruvu7y")
        .send()?;
    let body = res.text()?;
    println!("{}", body);
    let document = Html::parse_document(&body);
    let selector = Selector::parse("a").unwrap();
    for element in document.select(&selector) {
        let href = element.clone().value().attr("href").unwrap_or("");
        let txt = element.text().collect::<Vec<_>>().join("");
        let bytes = txt.bytes().collect::<Vec<_>>();
        println!("{}", href);
        // Decode the byte array using the Windows-1251 encoding
        let (html, _, _) = WINDOWS_1251.decode(&bytes);

        println!("text: {}", html);
        let utf8_html = UTF_8.encode(&html).0;
        println!("utf-8: {}", String::from_utf8_lossy(&utf8_html));
    }
    Ok(())
}
pub fn parse_details(html: &str) -> Result<(), Box<dyn std::error::Error>> {
    let document = Html::parse_document(&html);
    let mut selector = Selector::parse("div.phone").unwrap();
    for element in document.select(&selector) {
        let txt = element.text().collect::<Vec<_>>().join("");
        println!("here is the phone: {}", txt);
    }
    selector = Selector::parse("ul.dilarData").unwrap();
    for element in document.select(&selector) {
        let txt = element.text().collect::<Vec<_>>().join("_");
        let lines = txt.lines();
        for l in lines{
            if l.contains("_"){
                let v = l.split("_").collect::<Vec<&str>>();
                if v.len() >= 3{
                    info!("{}: {}", v[1], v[2]);
                }
                
            }
            
        }
        println!("dealer data: {}", txt.trim());
    }
    selector = Selector::parse("span.advact").unwrap();
    for element in document.select(&selector) {
        let txt = element.text().collect::<Vec<_>>().join(" ");
        println!("view counter: {}", txt.trim());
    }
    selector = Selector::parse("span#details_price").unwrap();
    for element in document.select(&selector) {
        let txt = element.text().collect::<Vec<_>>().join("_");
        println!("the price is: {}", txt.trim());
    }
    selector = Selector::parse("div.title").unwrap();
    for element in document.select(&selector) {
        let txt = element.text().collect::<Vec<_>>().join("_");
        println!("make and model: {}", txt.trim());
    }
    selector = Selector::parse("div[style*=\"margin-bottom:5px;\"]").unwrap();
    let divs = document.select(&selector);

    for div in divs {
        println!("{}", div.text().collect::<String>());
    }
    return Ok(());
}

pub fn get_found_result(html: &str) -> Result<String, Box<dyn std::error::Error>> {
    let fragment = Html::parse_document(&html);
    let selector = Selector::parse("meta[name=description]").unwrap();
    let description = fragment
        .select(&selector)
        .next()
        .unwrap()
        .value()
        .attr("content")
        .unwrap()
        .to_string();
    info!("description: {}", description);
    Ok(description)
}

pub fn get_links(html: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let document = Html::parse_document(&html);
    let selector = Selector::parse("a.pageNumbers").unwrap();
    let mut links = vec![];
    for element in document.select(&selector) {
        let txt = element.value().attr("href").unwrap_or("");
        if links.contains(&txt.to_string()) {
            continue;
        } else {
            links.push(txt.to_string());
        }
    }
    return Ok(links);
}

fn extract_price(html: &str) -> Option<(u32, Currency)> {
    let fragment = Html::parse_fragment(html);
    let selector = Selector::parse("span.price").unwrap();
    let element = match fragment.select(&selector).next() {
        Some(e) => e,
        None => return None, // return None if no <a> element is found
    };
    let price_element = element.text().collect::<Vec<_>>().join("");
    let contains_numeric = price_element.chars().any(|c| c.is_numeric());
    if !contains_numeric {
        return Some((0, Currency::BGN));
    }
    let v = price_element.replace(" ", "");
    let v1 = v.replace("&nbsp;", "");
    let (price_str, currency) = if v1.contains("USD") {
        (v1.trim_end_matches("USD"), Currency::USD)
    } else if v1.contains("EUR") {
        (v1.trim_end_matches("EUR"), Currency::EUR)
    } else {
        (v1.trim_end_matches("лв."), Currency::BGN)
    };
    println!("PRICE is: {} {:?}", price_str, currency);
    let price = match price_str.parse::<f32>() {
        Ok(p) => p.floor() as u32,
        Err(_) => return None, // return None if the string cannot be parsed as u32
    };
    Some((price, currency))
}

fn extract_adv_value(html: &str) -> Option<String> {
    let fragment = Html::parse_fragment(html);
    let selector = Selector::parse("a").unwrap();
    let element = match fragment.select(&selector).next() {
        Some(e) => e,
        None => return None, // return None if no <a> element is found
    };
    let href_attr = match element.value().attr("href") {
        Some(attr) => attr,
        None => return None, // return None if no href attribute is found
    };
    let adv_value = href_attr
        .split('&')
        .find(|s| s.starts_with("adv="))?
        .split('=')
        .last()?;
    Some(adv_value.to_owned())
}

pub fn search() -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.1 Safari/605.1.15")
        .build()?;
    let mut form_data = HashMap::new();
    form_data.insert("rub_pub_save", 1.to_string());
    form_data.insert("act", 3.to_string());
    form_data.insert("rub", 1.to_string());
    form_data.insert("f5", "Mercedes-Benz".to_owned());
    form_data.insert("f6", "C".to_owned());
    form_data.insert("f10", 2010.to_string());
    form_data.insert("f11", 2011.to_string());
    // form_data.insert("f12", "Бензин".to_owned());
    // form_data.insert("f13", "Автоматична".to_owned());
    // form_data.insert("f88", 1.to_string());
    // form_data.insert("f92", 1.to_string());
    // form_data.insert("f102", 1.to_string());
    let response = client
        .post("https://www.mobile.bg/pcgi/mobile.cgi")
        .form(&form_data)
        .send()?;

    let body = response.bytes().unwrap().to_vec();

    // Decode the byte array using the Windows-1251 encoding
    let (html, _, _) = WINDOWS_1251.decode(&body);

    // Convert the decoded text to UTF-8
    let utf8_html = UTF_8.encode(&html).0;
    let response = String::from_utf8_lossy(&utf8_html);
    return Ok(response.to_string());
}

pub fn search_form_data(input: &SearchRequest) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.1 Safari/605.1.15")
        .build()?;
    let body: Vec<u8> = client
        .post(SEARCH_URL)
        .form(&input.to_form_data())
        .send()?
        .bytes()
        .unwrap()
        .to_vec();
    let (html, _, _) = WINDOWS_1251.decode(&body);
    let utf8_html = UTF_8.encode(&html).0;
    let response = String::from_utf8_lossy(&utf8_html);
    return Ok(response.to_string());
}

pub fn get_pages(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.1 Safari/605.1.15")
        .build()?;
    let https_url = format!("https:{}", url);
    let body: Vec<u8> = client.get(&https_url).send()?.bytes().unwrap().to_vec();
    // Decode the byte array using the Windows-1251 encoding
    let (html, _, _) = WINDOWS_1251.decode(&body);
    // Convert the decoded text to UTF-8
    let utf8_html = UTF_8.encode(&html).0;
    let response = String::from_utf8_lossy(&utf8_html);
    return Ok(response.to_string());
}

pub fn get_vehicles_prices(html: &str) -> Vec<VehiclePrice> {
    let document = Html::parse_document(&html);
    let selector = Selector::parse("table.tablereset").unwrap();
    println!("found {} elements", document.select(&selector).count());
    let mut vehicle_prices = vec![];
    for element in document.select(&selector) {
        let prices = extract_price(&element.inner_html());
        let adv = extract_adv_value(&element.inner_html());
        let make_and_mode = make_and_mode(&element, HashMap::new());
        if adv.is_some() && prices.is_some() && make_and_mode.is_some() {
            let (make, model) = make_and_mode.unwrap();
            let (price, currency) = prices.unwrap();
            let mut vehicle_price = VehiclePrice
                ::new(adv.unwrap(), make, model, price, currency);
            vehicle_price.promoted = is_top_or_vip(&element);
            vehicle_price.sold = is_sold(&element);
            let (year, millage) = get_milllage_and_year(&element, vehicle_price.promoted);
            vehicle_price.year = year as u16;
            vehicle_price.millage = millage;
            vehicle_prices.push(vehicle_price);
        }
    }
    vehicle_prices
}

fn make_and_mode(element: &ElementRef, models: HashMap<&str, Vec<&str>>) -> Option<(String, String)> {
    let selector = Selector::parse("td.valgtop a.mmm").unwrap();
    for e in element.select(&selector) {
        let inner_html = e.inner_html();
        println!("Inner HTML: {}", inner_html);
        let strings = inner_html.split_ascii_whitespace().collect::<Vec<&str>>();
        if strings.is_empty() || strings.len() < 2{
            continue;
        }
        if models.is_empty() {
            info!("Make: {}, Model: {}", strings[0], strings[1]);
            return Some((strings[0].to_string(), strings[1].to_string()));
        }
   }    
    
    return None;
}

fn is_top_or_vip(element: &ElementRef) -> bool {
    let top = vec!["top", "vip"];
    for value in top {
        let filter = format!(r#"img[alt="{}"][class="noborder"]"#, value);
        let selector = Selector::parse(&filter).unwrap();
        let img_element_exists = element.select(&selector).next().is_some();
        if img_element_exists {
            return true;
        }
    }

    return false;
}

fn is_sold(element: &ElementRef) -> bool {
    let filter = r#"img"#;
    let selector = Selector::parse(&filter).unwrap();
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

fn get_milllage_and_year(element: &ElementRef, is_promoted: bool) -> (u32, u32) {
    let filter = match is_promoted {
        true => r#"td[colspan="3"]"#,
        false => r#"td[colspan="4"]"#,
    };

    let selector = Selector::parse(&filter).unwrap();
    let mut txt = element.select(&selector).next().unwrap().inner_html();
    txt = extract_ascii_latin(&txt);
    let numbers = extract_numbers(&txt);
    numbers
}

#[cfg(test)]
mod test {
    use crate::mobile_scraper::model::VehiclePrice;

    use super::model::MetaHeader;
    use super::*;
    use std::fs;
    use std::io::Result;

    fn read_file_from_resources(filename: &str) -> Result<String> {
        let path = format!("resources/{}", filename);
        fs::read_to_string(path)
    }

    #[test]
    fn test_read_meta_data() {
        let content = read_file_from_resources("found_13.html").unwrap();
        let meta_content = get_found_result(&content).unwrap();
        let meta = MetaHeader::from_string(&meta_content);
        assert_eq!(meta.make, "Skoda");
        assert_eq!(meta.model, "Octavia");
        assert_eq!(meta.min_price, 2300);
        assert_eq!(meta.max_price, 9999);
        assert_eq!(meta.total_number, 13);
    }

    #[test]
    fn test_read_links() {
        let content = read_file_from_resources("found_13.html").unwrap();
        let links = get_links(&content).unwrap();
        assert_eq!(links.len(), 0);
    }

    #[test]
    fn test_extract_adv_value() {
        let html = r#"<a href="//www.mobile.bg/pcgi/mobile.cgi?act=4&amp;adv=11677834464646619&amp;slink=rwjxia" class="mmm">Skoda Octavia 1.6i</a>"#;
        assert_eq!(
            extract_adv_value(html),
            Some("11677834464646619".to_owned())
        );

        let html = r#"<a href="//www.mobile.bg/pcgi/mobile.cgi?act=4&amp;slink=rwjxia" class="mmm">Skoda Octavia 1.6i</a>"#;
        assert_eq!(extract_adv_value(html), None);

        let html = r#"<p>Some text</p>"#;
        assert_eq!(extract_adv_value(html), None);
    }
    #[test]
    fn test_collect_all_prices() {
        let html = read_file_from_resources("sold.html").unwrap();
        let document = Html::parse_document(&html);
        let selector = Selector::parse("table.tablereset").unwrap();
        println!("found {} elements", document.select(&selector).count());
        for element in document.select(&selector) {
            let prices = extract_price(&element.inner_html());
            let adv = extract_adv_value(&element.inner_html());
            let make_and_mode = make_and_mode(&element, HashMap::new());
        if adv.is_some() && prices.is_some() && make_and_mode.is_some() {
            let (make, model) = make_and_mode.unwrap();
            let (price, currency) = prices.unwrap();
            let mut vehicle_price = VehiclePrice
                ::new(adv.unwrap(), make, model, price, currency);
                vehicle_price.promoted = is_top_or_vip(&element);
                vehicle_price.sold = is_sold(&element);
                let (year, millage) = get_milllage_and_year(&element, vehicle_price.promoted);
                vehicle_price.year = year as u16;
                vehicle_price.millage = millage;
                println!("vehicle_price: {:?}", vehicle_price);
            }
        }
    }

    #[test]
    fn test_is_top() {
        let html = read_file_from_resources("found_13.html").unwrap();
        let document = Html::parse_document(&html);
        let selector = Selector::parse("table.tablereset").unwrap();
        println!("found {} elements", document.select(&selector).count());
        for element in document.select(&selector) {
            let prices = extract_price(&element.inner_html());
            let adv = extract_adv_value(&element.inner_html());
            if adv.is_some() && prices.is_some() {
                println!("adv: {:?}, price: {:?}", adv, prices);
            }
        }
    }
}
