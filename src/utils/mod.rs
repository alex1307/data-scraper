use std::{
    error::Error,
    fs::File,
    io::{BufWriter, Write},
    thread,
    time::{Duration, SystemTime},
};

use chrono::{Local, NaiveDate};
use log::{error, info};
use serde::Serialize;

use crate::{
    config::links::ConfigData, model::traits::Header, DATE_FORMAT, DETAILS_URL, INIT_LOGGER,
    LISTING_URL,
};

pub fn configure_log4rs(file: &str) {
    INIT_LOGGER.call_once(|| {
        log4rs::init_file(file, Default::default()).unwrap();
        info!("SUCCESS: Loggers are configured with dir: _log/*");
    });
}

pub fn listing_url(slink: &str, page_number: &str) -> String {
    format!("{}&slink={}&f1={}", LISTING_URL, slink, page_number)
}

pub fn details_url(slink: &str, adv: &str) -> String {
    format!("{}&slink={}&adv={}", DETAILS_URL, slink, adv)
}

pub fn wait(min: u64, max: u64) {
    let wait_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        % max
        + min; // min 3 sec, max 10 sec
    thread::sleep(Duration::from_secs(wait_time));
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

pub fn bool_from_string(s: &str) -> Option<bool> {
    match s.trim().parse() {
        Ok(value) => Some(value),
        Err(_) => None,
    }
}

pub fn config_files<T: Serialize + Header>(source: &[ConfigData]) {
    source
        .iter()
        .for_each(|config| match create_empty_csv::<T>(&config.file_name) {
            Ok(_) => info!("Created file {}", config.file_name),
            Err(e) => error!("Error creating file {} {}", config.file_name, e),
        });
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
            return vec![format!("{}.{}", pattern.to_string(), ext.to_string())];
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
        file_names.push(format!("{}{}.{}", pattern, current_date.format(DATE_FORMAT), ext));
        current_date += chrono::Duration::days(1);
    }
    file_names
}

pub fn  subtract_vectors<T: PartialEq + Clone>(a: &[T], b: &[T]) -> Vec<T> {
    assert_eq!(a.len(), b.len(), "Vectors must have the same length");

    let result: Vec<T> = a
        .iter()
        .zip(b.iter())
        .filter(|(x, y)| x != y)
        .map(|(x, _)| x.clone())
        .collect();

    result
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

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
        assert_eq!(file_names[1], "test_".to_string() + &end_date+ ".csv");
    }

    #[test]
    fn test_extract() {
        let v1:Vec<String> = (0..100).map(|n| n.to_string()).collect();
        let v2: Vec<String> = (0..50).map(|n| (n*2).to_string()).collect();
        let h1: HashSet<String> = HashSet::from_iter(v1.iter().cloned());
        let h2:HashSet<String> = HashSet::from_iter(v2.iter().cloned());
        let diff = h1.difference(&h2).cloned().collect::<Vec<String>>();
        assert_eq!(diff.len(), 50);
    }
}
