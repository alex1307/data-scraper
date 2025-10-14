use futures::StreamExt;
use log::{error, info};
use rdkafka::{
    ClientConfig,
    consumer::{Consumer, StreamConsumer},
    message::Message,
};
use sqlx::PgPool;

use crate::{model::MetaDataModel::MetaDataKafkaRecord, writer::db_writer::db::MetaDBWriter};

/// Consume META messages from Kafka (topic: raptor.srp.meta)
/// Each message represents a filter context with url, filters, equipment, etc.
pub async fn consumeMetaTopic(broker: &str, group: &str, topic: &str) {
    info!("Starting META consumer for topic: {}", topic);

    // Kafka setup
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", group.to_owned() + "-meta")
        .set("bootstrap.servers", broker.to_string())
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("Meta consumer creation failed");

    consumer
        .subscribe(&[topic])
        .expect("Can't subscribe to META topic");

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in the environment");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to the database");
    let writer = MetaDBWriter::new(pool);

    let mut stream = consumer.stream();
    let mut counter = 0usize;

    while let Some(ev) = stream.next().await {
        match ev {
            Ok(msg) => {
                info!(
                    "Received META message: topic: {}, partition: {}, offset: {}",
                    msg.topic(),
                    msg.partition(),
                    msg.offset()
                );
                // Decode JSON payload
                if let Some(payload) = msg.payload_view::<str>() {
                    match payload {
                        Ok(json_str) => {
                            match serde_json::from_str::<MetaDataKafkaRecord>(json_str) {
                                Ok(meta_record) => {
                                    if let Err(e) = writer.write(meta_record).await {
                                        error!("DB write error: {}", e);
                                    } else {
                                        counter += 1;
                                        if counter % 10 == 0 {
                                            info!("META records written: {}", counter);
                                        }
                                    }
                                }
                                Err(e) => error!("Invalid META JSON: {}", e),
                            }
                        }
                        Err(e) => error!("UTF-8 error: {}", e),
                    }
                } else {
                    error!("Empty META message payload");
                }
            }
            Err(e) => error!("Kafka META error: {}", e),
        }
    }

    consumer.unsubscribe();
    info!("META consumer finished for topic: {}", topic);
}
