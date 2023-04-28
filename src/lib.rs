use config::links::ConfigData;
use log::{error, info};

pub mod config;
pub mod downloader;
pub mod model;
pub mod services;
pub mod writer;

use lazy_static::lazy_static;
use model::traits::Header;
use serde::Serialize;
use std::{
    sync::Once,
    thread,
    time::{Duration, SystemTime},
};
use writer::DataPersistance::create_empty_csv;

pub const LISTING_URL: &str = "//www.mobile.bg/pcgi/mobile.cgi?act=3";
pub const DETAILS_URL: &str = "//www.mobile.bg/pcgi/mobile.cgi?act=4&adv={}&slink={}";

lazy_static! {
    static ref INIT_LOGGER: Once = Once::new();
}

pub fn configure_log4rs() -> () {
    INIT_LOGGER.call_once(|| {
        log4rs::init_file("config/log4rs.yml", Default::default()).unwrap();
        info!("SUCCESS: Loggers are configured with dir: _log/*");
    });
}

pub fn listing_url(slink: &str, page_number: u32) -> String {
    format!(
        "{}{}",
        LISTING_URL,
        format!("&slink={}&f1={}", slink, page_number)
    )
}

pub fn details_url(slink: &str, adv: &str) -> String {
    format!("{}{}", DETAILS_URL, format!("&slink={}&adv={}", slink, adv))
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

pub fn bool_from_string(s: &str) -> Option<bool> {
    match s.trim().parse() {
        Ok(value) => Some(value),
        Err(_) => None,
    }
}

pub fn config_files<T: Serialize + Header>(source: &Vec<ConfigData>) {
    source
        .iter()
        .for_each(|config| match create_empty_csv::<T>(&config.file_name) {
            Ok(_) => info!("Created file {}", config.file_name),
            Err(e) => error!("Error creating file {} {}", config.file_name, e),
        });
}

pub mod stream_utils {

    use futures::{stream, Stream};
    use tokio::sync::mpsc;
    use tokio_stream::StreamExt;

    pub fn convert_mpsc_to_stream<T>(rx: &mut mpsc::Receiver<T>) -> impl Stream<Item = T> + '_ {
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
