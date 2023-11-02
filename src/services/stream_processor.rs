use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use crossbeam_channel::Receiver;

use crate::model::enums::Payload;
use crate::utils::helpers::crossbeam_utils::to_stream;
use futures::stream::StreamExt;
use log::{error, info};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;

use crate::model::traits::{Header, Identity};
use crate::services::file_processor;

pub struct Executor {
    pub file_name: String,
    pub counter: Arc<AtomicUsize>,
}

impl Executor {
    pub fn new(file_name: String, counter: Arc<AtomicUsize>) -> Self {
        Executor { file_name, counter }
    }
}

pub async fn process<
    T: Identity
        + Clone
        + Header
        + Debug
        + DeserializeOwned
        + Serialize
        + From<HashMap<String, String>>,
>(
    rx: &mut Receiver<Payload<HashMap<String, String>>>,
    file_name: &str,
    counter: &mut Arc<AtomicUsize>,
) {
    let thread_id = std::thread::current().id();
    info!(
        "start stream for file: {}, thread id: {:?} and counter: {}",
        file_name,
        thread_id,
        counter.load(std::sync::atomic::Ordering::SeqCst)
    );
    let stream = Box::pin(to_stream(rx));
    let mut processor: file_processor::DataProcessor<T> =
        file_processor::DataProcessor::from_files(vec![file_name]);
    let mut values = vec![];
    futures::pin_mut!(stream);
    while let Some(payload) = stream.next().await {
        if file_name.contains("errors_") {
            info!("Error processing: {:?}", payload);
        }
        match payload {
            Payload::Data(data) => {
                for m in data {
                    let value = T::from(m);
                    values.push(value);
                }
            }
            Payload::Value(data) => {
                let value = T::from(data);
                values.push(value);
            }
            Payload::Empty => continue,
            Payload::Error(data) => {
                error!("Error processing: {:?}", data);
                values.push(T::from(data));
            }
            Payload::Done => {
                info!("Done processing");
                counter.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
            }
        }

        if values.len() >= 20 {
            info!("Before processing: {:?}", values.len());
            processor.process(&values, None);
            values.clear();
            info!("After processing: {:?}", values.len());
        }
        if counter.load(std::sync::atomic::Ordering::SeqCst) == 0 {
            processor.process(&values, None);
            break;
        }
    }

    if !values.is_empty() {
        processor.process(&values, None);
    }
    info!(
        "Stream has finished for file: {}, thread id: {:?} and counter: {}",
        file_name,
        thread_id,
        counter.load(std::sync::atomic::Ordering::SeqCst)
    );
}
