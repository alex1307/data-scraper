extern crate scraper;

use log::info;
use reqwest;
use scraper::{Html, Selector};

use crate::{
    kafka::{
        broker,
        KafkaProducer::{create_producer, encode_message, send_message},
        EUR_EXCHANGE_RATE_TOPIC,
    },
    model::enums::Currency,
    protos::vehicle_model::EurExchangeRate,
    CREATED_ON,
};

pub async fn sync_exchange_rates() {
    let currencies = vec![Currency::CHF, Currency::BGN, Currency::PLN, Currency::EUR];
    let producer = create_producer(&broker());
    for currency in currencies {
        let rate = getRates(currency).await.unwrap();
        let encoded = encode_message(&rate).unwrap();
        info!("Rate: {:?}, encoded len: {}", rate, encoded.len());
        send_message(&producer, &EUR_EXCHANGE_RATE_TOPIC, encoded).await;
    }
}

pub async fn getRates(currency: Currency) -> Result<EurExchangeRate, String> {
    // Fetch the HTML content from the URL
    let url = format!(
        "https://www.xe.com/currencyconverter/convert/?Amount=1&From={}&To=EUR",
        currency.to_string()
    );
    let html = reqwest::get(&url)
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    let document = Html::parse_document(&html);

    // Create a selector for the <p> elements
    let a_element = Selector::parse("tr").unwrap();
    // Create a selector for the <td> elements inside <tr>
    let link = format!(
        "/currencyconverter/convert/?Amount=1&amp;From={}&amp;To=EUR",
        currency.to_string()
    );

    for tr_element in document.select(&a_element) {
        log::info!("tr_element: {:?}", tr_element.inner_html());
        if tr_element.inner_html().contains(&link) {
            let td_elements = Selector::parse("td").unwrap();
            let rate = tr_element
                .select(&td_elements)
                .nth(1)
                .unwrap()
                .inner_html()
                .chars()
                .filter(|c| c.is_ascii_digit() || c == &'.')
                .collect::<String>();
            println!("Rate: {}", rate);

            return Ok(EurExchangeRate {
                currency: currency.to_string(),
                eur_exchange_rate: rate.parse().unwrap(),
                created_on: CREATED_ON.to_string(),
            });
        }
    }

    Err("Exchange rate not found".to_string())
}

#[cfg(test)]
mod tests {
    use log::info;

    use super::*;
    use crate::{model::enums::Currency, utils::helpers::configure_log4rs, LOG_CONFIG};

    #[tokio::test]
    async fn test_CHF_getRates() {
        let rate = getRates(Currency::CHF).await.unwrap();
        assert_eq!(rate.currency, "CHF");
        assert_eq!(rate.eur_exchange_rate, 1.02354);
    }

    #[tokio::test]
    async fn test_PLN_getRates() {
        configure_log4rs(&LOG_CONFIG);
        let rate: EurExchangeRate = getRates(Currency::PLN).await.unwrap();
        assert_eq!(rate.currency, "PLN");
        info!("Rate: {:?}", rate);
        assert_eq!(rate.eur_exchange_rate, 0.233358);
    }

    #[tokio::test]
    async fn test_BGN_getRates() {
        configure_log4rs(&LOG_CONFIG);
        let rate: EurExchangeRate = getRates(Currency::BGN).await.unwrap();
        assert_eq!(rate.currency, "BGN");
        info!("Rate: {:?}", rate);
        assert_eq!(rate.eur_exchange_rate, 0.511292);
    }
}
