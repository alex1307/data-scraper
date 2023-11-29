use log::info;
use scraper::{Html, Selector};

fn get_listed_vehicles(source: &str) -> Vec<String> {
    let mut vehicles = vec![];
    let document = Html::parse_document(&source);
    let vehicle_selector = Selector::parse("li > div.search-row").unwrap();
    let mut counter = 0;
    for element in document.select(&vehicle_selector) {
        counter += 1;
        let inner_text = element.inner_html();
        vehicles.push(inner_text);
    }
    info!("counter: {}", counter);
    vehicles
}

fn get_total_number(source: &str) -> u32 {
    let document = Html::parse_document(&source);
    let selector = Selector::parse("li.c-breadcrumb").unwrap();
    let mut total_number = 0;
    for element in document.select(&selector) {
        let text = element.text().collect::<Vec<_>>().join(" ");
        if text.contains("Results") {
            info!("Found element text: {}", text);
            total_number = text
                .chars()
                .filter(|&c| c.is_numeric())
                .collect::<String>()
                .parse::<u32>()
                .unwrap_or(0);
            break;
        }
    }
    total_number
}
#[cfg(test)]
mod car_gr_test_suit{
    use std::collections::HashMap;

    use log::info;

    use crate::{scraper::ScraperTrait::Scraper, utils::helpers::configure_log4rs, LOG_CONFIG, helpers::CarGrHTMLHelper::{get_listed_vehicles, get_total_number}};


    #[tokio::test]
    async fn get_listes_vehicles_test() {
        configure_log4rs(&LOG_CONFIG);
        let url = "https://www.car.gr/classifieds/cars/?";
        let mut params = HashMap::new();
        params.insert("lang".to_owned(), "en".to_owned());
        params.insert("category=15001".to_owned(), "15001".to_owned());
        params.insert("price-from".to_owned(), "25000".to_owned());
        params.insert("price-to".to_owned(), "30000".to_owned());
        params.insert("registration-from".to_owned(), "2010".to_owned());
        params.insert("registration-to".to_owned(), "2011".to_owned());
        //params.insert("created".to_owned(), ">1".to_owned());
        let scraper = Scraper::new(url, "pg".to_owned(), 250);
        let url = scraper.search_url(None, params, 1);
        let html = scraper.html_search(&url, None).await.unwrap();
        //info!("html: {}", html);
        let data = get_listed_vehicles(&html);
        assert_eq!(data.len(), 24);
        let total_number = get_total_number(&html);
        assert_eq!(total_number, 122);
    }

}