use regex::Regex;

use super::model::MetaHeader;

pub fn extract_numbers(input: &str) -> (u32, u32) {
    if input.is_empty() {
        return (0, 0);
    }

    if input.trim().contains("няма намерени") {
        return (0, 0);
    }

    if !input.contains("от общо") {
        return (0, 0);
    }

    let re = Regex::new(r"\d+").unwrap();
    let numbers: Vec<u32> = re.find_iter(input).map(|mat| mat.as_str().parse().unwrap()).collect();
    
    let n = numbers[1];
    let k = numbers[2];
    
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
            make: "".to_string(),
            model: "".to_string(),
            min_price: 0,
            max_price: 0,
            total_number: 0,
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
    let total_number = split[3].replace(" ", "").parse::<u16>().unwrap_or(0);

    MetaHeader {
        make: make.to_string(),
        model: model.to_string(),
        min_price: min,
        max_price: max,
        total_number,
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
