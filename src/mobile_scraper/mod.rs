pub mod model;
pub mod mobile_utils;

use encoding_rs::{UTF_8, WINDOWS_1251};
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;

use self::model::SearchRequest;

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

pub fn get_found_result(html: &str)-> Result<String, Box<dyn std::error::Error>> {
    let fragment = Html::parse_document(&html);
    let selector = Selector::parse("meta[name=description]").unwrap();
    let description = fragment.select(&selector).next().unwrap().value().attr("content").unwrap().to_string();
    Ok(description)
}

pub fn get_links(html: &str)-> Result<Vec<String>, Box<dyn std::error::Error>> {
    let document = Html::parse_document(&html);
    let selector = Selector::parse("a.pageNumbers").unwrap();
    let mut links = vec![];
    for element in document.select(&selector) {
        let txt = element.value().attr("href").unwrap_or("");
        if links.contains(&txt.to_string()) {
            continue;
        }else {
            links.push(txt.to_string());
        }
    }
    return Ok(links)
}

pub fn parse_list(html: &str) -> Result<(), Box<dyn std::error::Error>> {
    let document = Html::parse_document(&html);
    let mut selector = Selector::parse(r#"input[type="hidden"][name="slink"]"#).unwrap();
    for element in document.select(&selector) {
        let txt = element.value().attr("value").unwrap_or("");
        println!("slink: {}", txt);
    }

    selector = Selector::parse("td.algright.valgtop").unwrap();
    let mut total = 0.0;
    for element in document.select(&selector) {
        let txt = element.text().collect::<Vec<_>>().join("");
        let price: Vec<&str> = txt.trim().split("лв.").collect();
        let s_no_space = price[0].trim().replace(" ", "");
        if s_no_space.is_empty() {
            continue;
        }
        println!("converting string to float...: {}", s_no_space.clone());
        let i = s_no_space.parse::<f32>().unwrap();
        println!("converted : {}", i.clone());
        total += i;
        println!("price: {}", price[0]);
        if price.len() == 2 && "Цената е без ДДС" == price[1]{
            println!("price: {}", price[1]);
            total += i*0.2;
        }
    }
    println!("total: {}", total);
    println!("avg: {}", total/20.0);
    return Ok(());
    
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
    let response = client.post("https://www.mobile.bg/pcgi/mobile.cgi")
        .form(&form_data)
        .send()?;
    
    let body = response.bytes().unwrap().to_vec();
    
    // Decode the byte array using the Windows-1251 encoding
    let (html, _, _) = WINDOWS_1251.decode(&body);

    // Convert the decoded text to UTF-8
    let utf8_html = UTF_8.encode(&html).0;
    let response = String::from_utf8_lossy(&utf8_html);
    println!("{}", response);
    return Ok(response.to_string());
}

pub fn open_link(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.1 Safari/605.1.15")
        .build()?;
    let response = client.get(url).send()?;
    let body = response.bytes().unwrap().to_vec();
    
    // Decode the byte array using the Windows-1251 encoding
    let (html, _, _) = WINDOWS_1251.decode(&body);

    // Convert the decoded text to UTF-8
    let utf8_html = UTF_8.encode(&html).0;
    let response = String::from_utf8_lossy(&utf8_html);
    println!("{}", response);
    return Ok(response.to_string());
}

pub fn search_form_data(url: &str, input: &SearchRequest) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.1 Safari/605.1.15")
        .build()?;
    let form_data = input.to_form_data();
    let response = client.post(url)
        .form(&form_data)
        .send()?;
    
    let body = response.bytes().unwrap().to_vec();
    
    // Decode the byte array using the Windows-1251 encoding
    let (html, _, _) = WINDOWS_1251.decode(&body);

    // Convert the decoded text to UTF-8
    let utf8_html = UTF_8.encode(&html).0;
    let response = String::from_utf8_lossy(&utf8_html);
    println!("{}", response);
    return Ok(response.to_string());
}


#[cfg(test)]
mod test{
    use super::*;
    use super::model::MetaHeader;
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
        assert_eq!(links.len(), 1);
    }

}