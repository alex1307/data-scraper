pub mod merge;
use std::{
    error::Error,
    fs::File,
    io::{BufWriter, Write},
    thread,
    time::{Duration, SystemTime},
};

use log::{error, info};
use serde::Serialize;

use crate::{
    config::links::ConfigData, model::traits::Header, DETAILS_URL, INIT_LOGGER, LISTING_URL,
};

pub fn configure_log4rs() {
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

pub fn config_files<T: Serialize + Header>(source: &Vec<ConfigData>) {
    source
        .iter()
        .for_each(|config| match create_empty_csv::<T>(&config.file_name) {
            Ok(_) => info!("Created file {}", config.file_name),
            Err(e) => error!("Error creating file {} {}", config.file_name, e),
        });
}

pub mod crossbeam_utils {
    use std::thread;

    use chrono::Duration;
    use crossbeam::channel::{Receiver, TryRecvError};
    use crossbeam_channel::RecvError;
    use futures::Stream;
    use log::info;

    pub fn to_stream<T>(rx: &mut Receiver<T>) -> impl Stream<Item = T> + '_ {
        async_stream::stream! {
            loop {
                match rx.recv_timeout(Duration::seconds(12).to_std().unwrap()) {
                    Ok(item) => yield item,
                    _ => break,
                }
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
