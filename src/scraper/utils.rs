use std::collections::HashMap;
use std::fs;
use std::io::Result;

use regex::Regex;
use scraper::{ElementRef, Selector};

pub fn read_file_from(dirname: &str, filename: &str) -> Result<String> {
    let path = format!("{}/{}", dirname, filename);
    fs::read_to_string(path)
}

pub fn extract_integers(s: &str) -> Vec<u32> {
    s.split_whitespace()
        .filter_map(|s| s.parse::<u32>().ok())
        .collect()
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

pub fn extract_ascii_latin(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_ascii_alphanumeric() || c.is_ascii_whitespace())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_numbers() {
        let input = "1 - 5 от общо 5";
        let expected_output = (1, 5);
        assert_eq!(extract_numbers(input), expected_output);
    }
}
