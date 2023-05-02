use futures::Stream;
use futures_lite::StreamExt;
use log::info;
use merge_streams::MergeStreams;

pub async fn merge_mpsc_to_stream<T: std::fmt::Debug>(streams: Vec<impl Stream<Item = T> + '_>) {
    let s = streams.merge();
    s.for_each(|n| info!("-> {:?}", n)).await;
}
