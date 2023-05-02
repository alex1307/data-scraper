use futures_lite::future::block_on;
use futures_lite::{stream, StreamExt};
use merge_streams::MergeStreams;
fn main() {
    block_on(async {
        let a = stream::once(1);
        let b = stream::once(2);
        let c = stream::once(3);

        let s = vec![a, b, c].merge();
        let mut counter = 0;
        s.for_each(|n| counter += n).await;
        assert_eq!(counter, 6);
    })
}
