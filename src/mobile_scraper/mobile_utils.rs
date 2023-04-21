use regex::Regex;

use super::model::MetaHeader;

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

    return (n, k);
}

pub fn extract_ascii_latin(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_ascii_alphanumeric() || c.is_ascii_whitespace())
        .collect()
}

pub fn read_meta_data(raw: &str) -> MetaHeader {
    let meta = extract_ascii_latin(raw);
    let re = Regex::new(r" {2,}").unwrap();
    let split: Vec<&str> = re.split(&meta.trim()).collect();
    if split.len() <= 4 {
        return MetaHeader {
            timestamp: chrono::Local::now().timestamp().to_string(),
            meta_type: "NEW".to_string(),
            make: "".to_string(),
            model: "".to_string(),
            min_price: 0,
            max_price: 0,
            total_number: 0,
            created_on: chrono::Local::now().format("%Y-%m-%d").to_string(),
        };
    }

    let make_model: Vec<&str> = split[0].split_whitespace().collect();

    let (make, model) = if make_model.len() == 1 {
        (make_model[0], "")
    } else {
        (make_model[0], make_model[1])
    };

    let min = split[1].replace(" ", "").parse::<u32>().unwrap_or(0);
    let max = split[2].replace(" ", "").parse::<u32>().unwrap_or(0);
    let total_number = split[3].replace(" ", "").parse::<u32>().unwrap_or(0);

    MetaHeader {
        timestamp: chrono::Local::now().timestamp().to_string(),
        meta_type: "NEW".to_string(),
        make: make.to_string(),
        model: model.to_string(),
        min_price: min,
        max_price: max,
        total_number,
        created_on: chrono::Local::now().format("%Y-%m-%d").to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_numbers() {
        let input = "1 - 5 от общо 5";
        let expected_output = (5, 5);
        assert_eq!(extract_numbers(input), expected_output);
    }

    #[test]
    fn test_read_meta_data() {
        let raw = "Обяви за » Skoda Rapid» на цени започващи от 2 300 лв. до 9 999 лв. Избор от » 13 « предложения само в Mobile.bg";
        let meta = read_meta_data(raw);
        assert_eq!(meta.make, "Skoda");
        assert_eq!(meta.model, "Rapid");
        assert_eq!(meta.min_price, 2300);
        assert_eq!(meta.max_price, 9999);
        assert_eq!(meta.total_number, 13);
    }
}

// Path: src/mobile_scraper/mobile_utils.rs
// Compare this snippet from src/mobile_scraper/mod.rs:
//         println!("{}", div.text().collect::<String>());
//     }
//     return Ok(());
// }
//
// pub fn get_found_result(html: &str)-> Result<String, Box<dyn std::error::Error>> {
//     let fragment = Html::parse_document(&html);
//     let selector = Selector::parse("meta[name=description]").unwrap();
//     let description = fragment.select(&selector).next().unwrap().value().attr("content").unwrap().to_string();
//     Ok(description)
// }
