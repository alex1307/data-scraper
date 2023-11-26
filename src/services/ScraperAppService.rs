use std::{collections::HashMap, str::FromStr};

use log::{error, info};

use crate::{
    model::records::MobileRecord,
    scraper::{
        CarsBgScraper::CarsBGScraper,
        MobileBgScraper::MobileBGScraper,
        ScraperTrait::{LinkId, ScraperTrait},
    },
    services::ScraperService::{process, save},
    utils::helpers::create_empty_csv,
    CARS_BG_INSALE_FILE_NAME, MOBILE_BG_FILE_NAME,
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref MOBILE_BG_CRAWLER: MobileBGScraper =
        MobileBGScraper::new("https://www.mobile.bg/pcgi/mobile.cgi?", 250);
    pub static ref CARS_BG_CRAWLER: CarsBGScraper = CarsBGScraper::new("https://www.cars.bg", 250);
}

use super::ScraperService::start;
#[derive(Debug, Clone)]
pub enum Crawlers {
    CarsBG(String),
    MobileBG(String),
}

impl FromStr for Crawlers {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "cars.bg" => Ok(Crawlers::CarsBG(
                r#"https://www.mobile.bg/pcgi/mobile.cgi?"#.to_owned(),
            )),
            "cars_bg" => Ok(Crawlers::CarsBG(
                r#"https://www.mobile.bg/pcgi/mobile.cgi?"#.to_owned(),
            )),
            "cars" => Ok(Crawlers::CarsBG(
                r#"https://www.mobile.bg/pcgi/mobile.cgi?"#.to_owned(),
            )),
            "mobile.bg" => Ok(Crawlers::MobileBG(r#"https://www.cars.bg"#.to_owned())),
            "mobile_bg" => Ok(Crawlers::MobileBG(r#"https://www.cars.bg"#.to_owned())),
            "mobile" => Ok(Crawlers::MobileBG(r#"https://www.cars.bg"#.to_owned())),
            _ => Err("Invalid crawler".into()),
        }
    }
}

pub async fn lets_scrape(crawler: &str) -> Result<(), String> {
    let crawler = Crawlers::from_str(crawler)?;

    match crawler {
        Crawlers::CarsBG(_) => {
            let searches = searches(crawler.clone()).await;
            run_scraper(
                CARS_BG_CRAWLER.clone(),
                CARS_BG_INSALE_FILE_NAME.to_owned(),
                searches,
                "cars.bg",
            )
            .await
        }
        Crawlers::MobileBG(_) => {
            let searches = searches(crawler.clone()).await;
            run_scraper(
                MOBILE_BG_CRAWLER.clone(),
                MOBILE_BG_FILE_NAME.to_owned(),
                searches,
                "mobile.bg",
            )
            .await
        }
    }
}

async fn searches(cralwer: Crawlers) -> Vec<HashMap<String, String>> {
    match cralwer {
        Crawlers::CarsBG(_) => {
            let prices = [
                1000, 2000, 3000, 4000, 5000, 6000, 7000, 8000, 9000, 10_000, 11_000, 13_000,
                15_000, 18_000, 19_000, 21_000, 25_000, 30_000, 40_000, 50_000, 95_000,
            ];
            let mut map = HashMap::new();
            map.insert("subm".to_owned(), "1".to_owned());
            map.insert("add_search".to_owned(), "1".to_owned());
            map.insert("typeoffer".to_owned(), "1".to_owned());
            map.insert("last".to_owned(), "1".to_owned());
            map.insert("conditions[]".to_owned(), "1".to_owned());
            map.insert("yearFrom".to_owned(), "2004".to_owned());
            map.insert("steering_wheel".to_owned(), "1".to_owned());
            let mut searches = vec![];
            for i in 0..prices.len() - 2 {
                let mut price_filter = map.clone();
                price_filter.insert("priceFrom".to_owned(), prices[i].to_string());
                price_filter.insert("priceTo".to_owned(), (prices[i + 1]).to_string());
                searches.push(price_filter);
            }
            map.remove("priceTo");
            let mut most_expensive = map.clone();
            most_expensive.insert("priceFrom".to_owned(), "95000".to_owned());
            searches.push(most_expensive);
            searches
        }
        Crawlers::MobileBG(_) => mobile_bg_searches().await,
    }
}

async fn mobile_bg_searches() -> Vec<HashMap<String, String>> {
    let prices = [
        1000, 2000, 3000, 4000, 5000, 6000, 7000, 8000, 9000, 10_000, 11_000, 13_000, 15_000,
        18_000, 19_000, 21_000, 25_000, 30_000, 40_000, 50_000, 95_000,
    ];
    let mut meta_searches = vec![];
    let mut searches = vec![];
    let mut params = HashMap::new();
    params.insert("act".to_owned(), "3".to_owned());
    params.insert("f10".to_owned(), "2004".to_owned());
    params.insert("topmenu".to_string(), "1".to_string());
    params.insert("rub".to_string(), 1.to_string());
    params.insert("pubtype".to_string(), 1.to_string());
    params.insert("f20".to_string(), 7.to_string());
    for i in 0..prices.len() - 2 {
        let mut params = params.clone();
        params.insert("f7".to_owned(), prices[i].to_string());
        params.insert("f8".to_owned(), (prices[i + 1]).to_string());
        meta_searches.push(params.clone());
    }
    let mut most_expensive = params.clone();
    params.remove("f8");
    most_expensive.insert("f7".to_owned(), "95000".to_owned());
    meta_searches.push(most_expensive);
    params.remove("f7");
    params.remove("f8");

    let mut sold_vehicles = params.clone();
    sold_vehicles.insert(
        "f94".to_string(),
        "1~%CA%E0%EF%E0%F0%E8%F0%E0%ED%5C%CF%F0%EE%E4%E0%E4%E5%ED".to_string(),
    );
    meta_searches.push(sold_vehicles);

    params.clear();
    params.insert("act".to_owned(), "3".to_owned());
    params.insert("rub".to_string(), 1.to_string());
    params.insert("pubtype".to_string(), 1.to_string());
    params.insert("topmenu".to_string(), "1".to_string());

    for search in meta_searches.clone() {
        let slink = MOBILE_BG_CRAWLER.slink(search.clone()).await.unwrap();
        params.insert("slink".to_owned(), slink.clone());
        info!("slink: {} for search: {:?}", slink, search);
        searches.push(params.clone());
    }
    searches
}

pub async fn run_scraper<T: ScraperTrait + Clone + Send>(
    scraper: T,
    file_name: String,
    searches: Vec<HashMap<String, String>>,
    source: &str,
) -> Result<(), String>
where
    T: 'static,
{
    let (mut link_producer, mut link_receiver) = tokio::sync::mpsc::channel::<LinkId>(250);
    let (mut record_producer, record_receiver) = tokio::sync::mpsc::channel::<MobileRecord>(250);
    if create_empty_csv::<MobileRecord>(&file_name).is_err() {
        error!("Failed to create file {}", file_name.clone());
    }
    let process_scraper = scraper.clone();
    let start_handler =
        tokio::spawn(async move { start(Box::new(scraper), searches, &mut link_producer).await });
    let process_handler = tokio::spawn(async move {
        process(process_scraper, &mut link_receiver, &mut record_producer).await
    });
    let cloned = source.to_owned();
    let save_to_file = tokio::spawn(async move { save(record_receiver, file_name, &cloned).await });

    if let (Ok(_), Ok(_), Ok(_)) = tokio::join!(start_handler, process_handler, save_to_file) {
        info!("All tasks completed successfully");
        Ok(())
    } else {
        error!("One or more tasks failed");
        Err("One or more tasks failed".into())
    }
}

#[cfg(test)]
mod app_test {
    use std::collections::HashMap;
    use std::str::FromStr;

    use log::info;

    use super::run_scraper;
    use crate::scraper::CarsBgScraper::CarsBGScraper;
    use crate::scraper::MobileBgScraper::MobileBGScraper;
    use crate::scraper::ScraperTrait::ScraperTrait;
    use crate::services::ScraperAppService::{Crawlers, CARS_BG_CRAWLER, MOBILE_BG_CRAWLER};
    use crate::utils::helpers::configure_log4rs;
    use crate::LOG_CONFIG;

    #[tokio::test]
    async fn test_run_carsbg_scraper() {
        configure_log4rs(&LOG_CONFIG);
        let mut params = HashMap::new();
        //subm=1&add_search=1&typeoffer=1&priceFrom=18000&priceTo=30000&yearFrom=2007&yearTo=2011&page=32
        params.insert("subm".to_owned(), "1".to_owned());
        params.insert("add_search".to_owned(), "1".to_owned());
        params.insert("typeoffer".to_owned(), "1".to_owned());
        params.insert("priceFrom".to_owned(), "29500".to_owned());
        params.insert("priceTo".to_owned(), "30000".to_owned());
        params.insert("yearFrom".to_owned(), "2010".to_owned());
        params.insert("yearTo".to_owned(), "2011".to_owned());
        let searches = vec![params];
        let scraper = CarsBGScraper::new("https://www.cars.bg", 250);
        run_scraper(
            scraper,
            "./resources/test-data/test.csv".to_owned(),
            searches,
            "cars.bg",
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_run_mobilebg_scraper() {
        configure_log4rs(&LOG_CONFIG);
        let mut params = HashMap::new();
        params.insert("act".to_owned(), "3".to_owned());
        params.insert("f10".to_owned(), "2004".to_owned());
        params.insert("topmenu".to_string(), "1".to_string());
        params.insert("rub".to_string(), 1.to_string());
        params.insert("pubtype".to_string(), 1.to_string());
        params.insert("f7".to_string(), 10000.to_string());
        params.insert(
            "f94".to_string(),
            "1~%CA%E0%EF%E0%F0%E8%F0%E0%ED%5C%CF%F0%EE%E4%E0%E4%E5%ED".to_string(),
        );

        let scraper = MobileBGScraper::new("https://www.mobile.bg/pcgi/mobile.cgi?", 250);
        let slink = scraper.slink(params.clone()).await.unwrap();
        params.clear();
        params.insert("act".to_owned(), "3".to_owned());
        params.insert("rub".to_string(), 1.to_string());
        params.insert("pubtype".to_string(), 1.to_string());
        params.insert("topmenu".to_string(), "1".to_string());
        params.insert("slink".to_owned(), slink.clone());
        let searches = vec![params];
        run_scraper(
            scraper,
            "./resources/test-data/test.csv".to_owned(),
            searches,
            "mobile.bg",
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_cars_bg_searches() {
        let crawler = Crawlers::from_str("cars.bg").unwrap();
        configure_log4rs(&LOG_CONFIG);
        let searches = super::searches(crawler).await;
        let mut total = 0;
        for search in searches {
            let total_number = CARS_BG_CRAWLER.total_number(search.clone()).await.unwrap();
            total += total_number;
            info!("total_number: {} for search: {:?}", total_number, search);
        }
        info!("total: {}", total);
    }

    #[tokio::test]
    async fn test_mobile_bg_searches() {
        let crawler = Crawlers::from_str("mobile.bg").unwrap();
        configure_log4rs(&LOG_CONFIG);
        let searches = super::searches(crawler).await;
        let mut total = 0;
        for search in searches {
            let total_number = MOBILE_BG_CRAWLER
                .total_number(search.clone())
                .await
                .unwrap();
            total += total_number;
            info!("total_number: {} for search: {:?}", total_number, search);
        }
        info!("total: {}", total);
    }
}
