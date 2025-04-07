use std::{fmt::Debug, sync::Mutex, time::Duration};

use futures::future::join_all;
use log::{debug, error, info};
use serde::Serialize;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    time::{sleep, timeout},
};

use lazy_static::lazy_static;
use uuid::Uuid;

#[cfg(feature = "kafka")]
use crate::kafka::{BASE_INFO_TOPIC, DETAILS_TOPIC, PRICE_TOPIC, broker};
#[cfg(feature = "kafka")]
use crate::writer::kafka_writer::kafka::KafkaProducer;

use crate::{
    model::{
        Search::Search,
        VehicleDataModel::{
            BaseVehicleInfo, BasicT, ChangeLogT, DetailedVehicleInfo, DetailsT, DownloadStatus,
            Price, PriceT, ScrapedListData,
        },
    },
    utils::files::{base_file_name, details_file_name, prices_file_name},
    writer::{
        flle_writer::file::FileWriter,
        sink::{FormatterType, Sink, SinkType},
    },
};

lazy_static! {
    pub static ref TOTAL_COUNT: Mutex<u32> = Mutex::new(0);
}

#[derive(Debug, Clone)]
pub struct ScraperService<T: ScraperTrait + Clone> {
    pub scraper: T,
    pub file_name: String,
}
