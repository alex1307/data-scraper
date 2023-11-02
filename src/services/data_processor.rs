use log::{error, info};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use tokio::sync::mpsc::Receiver;

use crate::{
    model::traits::Header,
    utils::helpers::create_empty_csv,
    writer::persistance::{MobileData, MobileDataWriter},
};

pub enum Message<T: Debug + Serialize + DeserializeOwned + Clone + Header> {
    Data(Vec<T>),
    Value(T),
    Stop,
}

pub struct Processor<T: Debug + Serialize + DeserializeOwned + Clone + Header> {
    receiver: Receiver<Message<T>>,
    data: Vec<T>,
    file: String,
    cache_size: u32,
}

impl<T: Debug + Serialize + DeserializeOwned + Clone + Header> Processor<T> {
    pub fn new(receiver: Receiver<Message<T>>, file: &str) -> Self {
        match create_empty_csv::<T>(file) {
            Ok(_) => info!("File {} created", file),
            Err(e) => error!("Error creating file {}: {}", file, e),
        }
        Self {
            receiver,
            data: vec![],
            file: file.to_string(),
            cache_size: 100,
        }
    }

    pub async fn handle(&mut self) {
        info!("Start handling messages");
        loop {
            let mut do_exit = false;
            match self.receiver.recv().await {
                Some(message) => match message {
                    Message::Data(data) => {
                        self.data.extend(data);
                    }
                    Message::Stop => {
                        do_exit = true;
                    }
                    Message::Value(_) => todo!(),
                },
                None => continue,
            }

            // Check if there are more messages available without blocking
            for message in self.receiver.try_recv().into_iter() {
                match message {
                    Message::Data(data) => {
                        self.data.extend(data);
                    }
                    Message::Stop => {
                        do_exit = true;
                    }
                    Message::Value(_) => todo!(),
                }
            }

            if self.data.len() >= self.cache_size as usize {
                let data = MobileData::Payload(self.data.clone());
                data.write_csv(&self.file, false).unwrap();
                info!("write {} records to {}", self.data.len(), &self.file);
                self.data.clear();
            }

            if do_exit {
                break;
            }
        }
        if !self.data.is_empty() {
            let data = MobileData::Payload(self.data.clone());
            data.write_csv(&self.file, false).unwrap();
            info!("write {} records to {}", self.data.len(), &self.file);
            self.data.clear();
        }
    }
}
