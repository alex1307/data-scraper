use std::sync::mpsc::Sender;

use async_trait::async_trait;
use tokio::sync::mpsc::Receiver;

#[async_trait]
pub trait CrawlerTrait<S, Req, Res> {
    async fn process_details(
        scraper: S,
        link_receiver: &mut Receiver<Req>,
        records_producer: &mut Sender<Res>,
    ) -> Result<(), String>;
}
