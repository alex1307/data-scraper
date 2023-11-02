use std::{
    error::Error,
    fs::{self, File},
    io::{BufWriter, Write},
};

use chrono::{Local, NaiveDate};
use log::{error, info};
use serde::Serialize;

use crate::{
    model::{enums::SaleType, traits::Header},
    DATE_FORMAT, DETAILS_URL, INIT_LOGGER,
};

pub fn configure_log4rs(file: &str) {
    INIT_LOGGER.call_once(|| {
        log4rs::init_file(file, Default::default()).unwrap();
        info!("SUCCESS: Loggers are configured with dir: _log/*");
    });
}

pub fn mobile_search_url(
    url: &str,
    source: &str,
    slink: &str,
    sale: SaleType,
    min: i32,
    max: i32,
) -> String {
    let mut params: Vec<String> = vec![];
    if DETAILS_URL == url {
        let slink_param = format!("slink={}", slink);
        let adv_param: String = format!("adv={}", source);
        params.push(slink_param);
        params.push(adv_param);
        let url_encoded_params = params.join("&");
        return format!("{}{}", url, url_encoded_params);
    }
    params.push("topmenu=1".to_string());
    params.push("rub=1".to_string());
    let pg = format!("f1={}", source);
    params.push(pg);
    if slink.trim() != "" {
        let slink_url = format!("slink={}", slink);
        params.push(slink_url);
        let url_encoded_params = params.join("&");
        return format!("{}{}", url, url_encoded_params);
    }

    if sale == SaleType::SOLD {
        params.push("f94=1~%CA%E0%EF%E0%F0%E8%F0%E0%ED%5C%CF%F0%EE%E4%E0%E4%E5%ED".to_string());
    } else if sale == SaleType::INSALE {
        params.push("f20=7".to_string());
    }

    if min > 0 {
        let min_price = format!("f7={}", min);
        params.push(min_price);
    }

    if max > 0 {
        let max_price = format!("f8={}", max);
        params.push(max_price);
    }

    let url_encoded_params = params.join("&");
    format!("{}{}", url, url_encoded_params)
}

pub fn create_empty_csv<T: Serialize + Header>(file_path: &str) -> Result<(), Box<dyn Error>> {
    let path = std::path::Path::new(file_path);
    if path.exists() {
        return Err(format!("File {} already exists.", file_path).into());
    }
    let line = T::header().join(","); // Convert the vector to a comma-separated string
    let file = File::create(file_path)?; // Create a new file for writing
    let mut writer = BufWriter::new(file);
    writer.write_all(line.as_bytes())?;
    writer.write_all(b"\r\n")?;
    writer.flush()?;
    Ok(())
}

pub mod crossbeam_utils {

    use chrono::Duration;
    use crossbeam::channel::Receiver;

    use futures::Stream;

    pub fn to_stream<T>(rx: &mut Receiver<T>) -> impl Stream<Item = T> + '_ {
        async_stream::stream! {
            while let Ok(item) = rx.recv_timeout(Duration::seconds(12).to_std().unwrap())  {
                yield item;
            }
        }
    }
}

pub mod stream_utils {

    use futures::{stream, Stream, StreamExt};
    use tokio::sync::mpsc::{self, Receiver};

    pub fn convert_mpsc_to_stream<T>(rx: &mut Receiver<T>) -> impl Stream<Item = T> + '_ {
        stream::unfold(rx, |rx| async move { rx.recv().await.map(|t| (t, rx)) })
    }

    pub fn join_mpsc_to_stream<T>(rx: &mut Vec<mpsc::Receiver<T>>) -> impl Stream<Item = T> + '_ {
        stream::unfold(rx, |rx| async move {
            let mut rx1 = rx.pop().unwrap();
            rx1.recv().await.map(|t| (t, rx))
        })
    }

    pub async fn message_consumer<S, M>(mut stream: S)
    where
        S: Stream<Item = M> + Unpin,
        M: std::fmt::Debug + Send + Sync,
    {
        while let Some(msg) = stream.next().await {
            // Do something with the message
            println!("{:?}", msg);
        }
    }
}

pub fn get_file_names(pattern: &str, from_date: &str, to_date: &str, ext: &str) -> Vec<String> {
    let start_date = match NaiveDate::parse_from_str(from_date, DATE_FORMAT) {
        Ok(date) => date,
        Err(e) => {
            error!("Invalid from/start date {}", e);
            return vec![format!("{}.{}", pattern, ext)];
        }
    };

    let end_date = match NaiveDate::parse_from_str(to_date, DATE_FORMAT) {
        Ok(date) => date,
        Err(e) => {
            error!("Invalid end/to date {}", e);
            Local::now().date_naive()
        }
    };

    let mut current_date = start_date;
    let mut file_names = vec![];
    while current_date <= end_date {
        file_names.push(format!(
            "{}{}.{}",
            pattern,
            current_date.format(DATE_FORMAT),
            ext
        ));
        current_date += chrono::Duration::days(1);
    }
    file_names
}

pub fn subtract_vectors<T: PartialEq + Clone>(a: &[T], b: &[T]) -> Vec<T> {
    assert_eq!(a.len(), b.len(), "Vectors must have the same length");

    let result: Vec<T> = a
        .iter()
        .zip(b.iter())
        .filter(|(x, y)| x != y)
        .map(|(x, _)| x.clone())
        .collect();

    result
}

pub fn read_file_from(dirname: &str, filename: &str) -> Result<String, std::io::Error> {
    let path = format!("{}/{}", dirname, filename);
    fs::read_to_string(path)
}

pub fn extract_integers(s: &str) -> Vec<u32> {
    s.split_whitespace()
        .filter_map(|s| s.parse::<u32>().ok())
        .collect()
}

pub fn extract_ascii_latin(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_ascii_alphanumeric() || c.is_ascii_whitespace())
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::scraper::mobile_bg::{get_pages, slink};

    use super::*;

    #[test]
    fn test_get_file_names() {
        let file_names = get_file_names("test_", "2020-01-01", "2020-01-03", "csv");
        assert_eq!(file_names.len(), 3);
        assert_eq!(file_names[0], "test_2020-01-01.csv");
        assert_eq!(file_names[1], "test_2020-01-02.csv");
        assert_eq!(file_names[2], "test_2020-01-03.csv");

        let file_names = get_file_names("test_", "", "", "csv");
        assert_eq!(file_names.len(), 1);
        assert_eq!(file_names[0], "test_.csv");
        let today = Local::now().date_naive();
        let yesterday = today - chrono::Duration::days(1);
        let from_date = yesterday.format(DATE_FORMAT).to_string();
        let end_date = today.format(DATE_FORMAT).to_string();
        let file_names = get_file_names("test_", &from_date, "", "csv");
        assert_eq!(file_names.len(), 2);
        assert_eq!(file_names[0], "test_".to_string() + &from_date + ".csv");
        assert_eq!(file_names[1], "test_".to_string() + &end_date + ".csv");
    }

    #[test]
    fn test_extract() {
        let v1: Vec<String> = (0..100).map(|n| n.to_string()).collect();
        let v2: Vec<String> = (0..50).map(|n| (n * 2).to_string()).collect();
        let h1: HashSet<String> = HashSet::from_iter(v1.iter().cloned());
        let h2: HashSet<String> = HashSet::from_iter(v2.iter().cloned());
        let diff = h1.difference(&h2).cloned().collect::<Vec<String>>();
        assert_eq!(diff.len(), 50);
    }
    #[test]
    fn test_slink() {
        configure_log4rs("config/loggers/dev_log4rs.yml");
        info!("test_slink");
        let url = mobile_search_url(crate::LISTING_URL, "1", "", SaleType::SOLD, 0, 5000);
        assert_eq!(
            url,
            "https://www.mobile.bg/pcgi/mobile.cgi?act=3&topmenu=1&rub=1&f1=1&f94=1~%CA%E0%EF%E0%F0%E8%F0%E0%ED%5C%CF%F0%EE%E4%E0%E4%E5%ED&f7=0&f8=5000"
        );
        let html = get_pages(&url).unwrap();
        // info!("content: {}", html);
        let slink = slink(&html);
        info!("slink: {}", slink);
    }
}
