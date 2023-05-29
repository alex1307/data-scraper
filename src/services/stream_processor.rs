use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use crossbeam_channel::Receiver;

use crate::model::enums::Payload;
use futures::stream::StreamExt;
use log::info;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;

use crate::model::traits::{Header, Identity};
use crate::services::file_processor;
use crate::utils::crossbeam_utils::to_stream;

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
    let stream = Box::pin(to_stream(rx));
    let mut processor: file_processor::DataProcessor<T> =
        file_processor::DataProcessor::from_file(file_name);
    let mut values = vec![];
    futures::pin_mut!(stream);
    while let Some(payload) = stream.next().await {
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
                info!("Error processing: {:?}", data);
                values.push(T::from(data));
            }
            Payload::Done => {
                info!("Done processing");
                counter.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
            }
        }

        if values.len() >= 20 {
            processor.process(&values, None);
            values.clear();
        }
        if counter.load(std::sync::atomic::Ordering::SeqCst) == 0 {
            processor.process(&values, None);
            break;
        }
    }

    if !values.is_empty() {
        processor.process(&values, None);
    }
}
