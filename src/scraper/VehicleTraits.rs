use crate::{
    helpers::{
        AutoUncleHelper::process_html as process_autouncle_html, MobileBgHTMLHelper::process_html,
    },
    model::{
        Search::Search,
        VehicleDataModel::{BasicT, DetailsT, PriceT, Vehicle},
    },
};
use async_trait::async_trait;
use log::error;
use rand::Rng;
use regex::Regex;
use scraper::{Html, Selector};
use std::{fmt::Debug, time::Duration};
use tokio::time::sleep;

use super::{MobileBgScraper::MobileBGScraper, Traits::Scraper};

#[async_trait]
pub trait VehicleScrapeTrait: Clone + Debug + Send + Sync + 'static {
    async fn process_listed_results(
        &self,
        search: Search,
        page: u32,
    ) -> Result<Vec<Vehicle>, String>;
    async fn get_html(&self, search: Search, page: u32) -> Result<String, String>;

    fn total_number(&self, page: &str) -> Result<u32, String>;

    fn get_number_of_pages(&self, total_number: u32) -> Result<u32, String>;

    fn get_timeout(&self) -> u64 {
        250
    }

    fn get_search_url(&self, search: Search, page: u32) -> String {
        if page == 1 {
            return search.url;
        }
        format!("{}&page={}", search.url, page)
    }

    fn process_html(&self, html: &str) -> Result<Vec<Vehicle>, String>;
}

#[async_trait]
impl VehicleScrapeTrait for MobileBGScraper {
    async fn get_html(&self, search: Search, page: u32) -> Result<String, String> {
        let url = self.get_search_url(search, page);
        self.parent
            .html_search(&url, Some("windows-1251".to_string()))
            .await
    }
    async fn process_listed_results(
        &self,
        search: Search,
        page: u32,
    ) -> Result<Vec<Vehicle>, String> {
        let html = self.get_html(search.clone(), page).await?;
        let vehicles = self.process_html(&html)?;
        Ok(vehicles)
    }

    fn process_html(&self, html: &str) -> Result<Vec<Vehicle>, String> {
        let vehicles: Vec<Vehicle> = process_html(html)
            .iter()
            .cloned()
            .map(|r| Vehicle {
                id: r.id.clone(),
                source: "mobile.bg".to_string(),
                make: r.make.clone(),
                model: r.model.clone(),
                title: r.title.clone(),
                currency: r.currency,
                price: r.price,
                mileage: r.mileage,
                year: r.year,
                location: Some(r.location()),
                engine: r.engine,
                gearbox: r.gearbox,
                power_ps: r.power_ps(),
                power_kw: r.power_kw(),
                url: r.url(),
                seller_name: r.name,
                seller_url: r.dealer_url,
                ..Default::default()
            })
            .collect();
        if vehicles.is_empty() {
            return Err("No vehicles found".to_string());
        }
        Ok(vehicles)
    }

    fn total_number(&self, html: &str) -> Result<u32, String> {
        let document = Html::parse_document(html);
        let selector = Selector::parse(r#"meta[name="description"]"#).unwrap();

        if let Some(element) = document.select(&selector).next() {
            if let Some(content) = element.value().attr("content") {
                let re = Regex::new(r"»\s*(\d+)\s*«").unwrap();
                if let Some(caps) = re.captures(content) {
                    if let Some(matched) = caps.get(1) {
                        let total_number = matched
                            .as_str()
                            .parse::<u32>()
                            .map_err(|_| "Failed to parse number from string".to_string());
                        return total_number;
                    }
                } else {
                    error!("Number not found");
                }
            }
        } else {
            error!("Total number not found");
        }

        Err("Number not found".to_string())
    }

    fn get_number_of_pages(&self, total_number: u32) -> Result<u32, String> {
        self.parent.get_number_of_pages(total_number)
    }

    fn get_search_url(&self, search: Search, page: u32) -> String {
        if page == 1 {
            search.url.replace("{page}", "")
        } else {
            search.url.replace("{page}", &format!("/p-{}", page))
        }
    }
    fn get_timeout(&self) -> u64 {
        250
    }
}

#[derive(Debug, Clone)]
pub struct AutouncleScraper {
    pub parent: Scraper,
}

impl AutouncleScraper {
    pub fn new(url: &str, page: &str, source: &str, wait_time_ms: u64) -> Self {
        AutouncleScraper {
            parent: Scraper::new(url, page.to_string(), source.to_string(), wait_time_ms),
        }
    }
}

#[async_trait]
impl VehicleScrapeTrait for AutouncleScraper {
    async fn process_listed_results(
        &self,
        search: Search,
        page: u32,
    ) -> Result<Vec<Vehicle>, String> {
        let html = self.get_html(search.clone(), page).await?;
        let vehicles = self.process_html(&html)?;

        let waiting_time_ms: u64 = rand::rng().random_range(3_000..5_000);
        log::info!("Sleeping for{} millis", waiting_time_ms);
        sleep(Duration::from_millis(waiting_time_ms as u64)).await;
        log::info!("keeping on...");
        Ok(vehicles)
    }

    fn process_html(&self, html: &str) -> Result<Vec<Vehicle>, String> {
        let vehicles: Vec<Vehicle> = process_autouncle_html(html)
            .iter()
            .cloned()
            .map(|r| Vehicle {
                id: r.car_id.clone(),
                source: self.parent.source.clone(),
                make: r.make(),
                model: r.model(),
                title: r.title(),
                currency: <dyn BasicT>::currency(&r),
                price: r.price().unwrap_or_default(),
                mileage: r.millage().unwrap_or_default(),
                year: r.year(),
                location: Some(r.location()),
                engine: r.engine,
                gearbox: r.gearbox(),
                power_ps: r.power_ps(),
                power_kw: r.power_kw(),
                url: r.url(),
                seller_name: r.seller_name(),
                seller_url: r.seller_url(),
                estimated_price: r.estimated_price(),
                cc: Some(r.cc()),
                equipment: Some(r.equipment()),
                range: r.range(),
                consumption_fuel: r.litter_fuel_consumption(),
                consumption_kw: r.kwh_fuel_consumption(),
                co2: Some(r.co2()),
                days_in_sale: r.days_in_sale(),
                ..Default::default()
            })
            .collect();
        if vehicles.is_empty() {
            return Err("No vehicles found".to_string());
        }
        Ok(vehicles)
    }
    async fn get_html(&self, search: Search, page: u32) -> Result<String, String> {
        let url = self.get_search_url(search, page);
        self.parent.html_search(&url, None).await
    }

    fn total_number(&self, html: &str) -> Result<u32, String> {
        let number_of_cars = match html.find(r#"\"numberOfCars\":"#) {
            Some(index) => {
                let index = index + r#"\"numberOfCars\":"#.len();
                let end_index = html[index..].find(',').unwrap();
                html[index..index + end_index].parse::<u32>().unwrap()
            }
            None => return Err("Not found".to_string()),
        };
        Ok(number_of_cars)
    }

    fn get_number_of_pages(&self, total_number: u32) -> Result<u32, String> {
        let number_of_pages = (total_number as f32 / 25.0).ceil() as u32;
        Ok(number_of_pages)
    }
}
