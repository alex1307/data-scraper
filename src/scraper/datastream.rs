use futures::{Stream, Future, task::{Context, Poll}, FutureExt, StreamExt};
use std::collections::HashMap;
use std::pin::Pin;
use std::vec::IntoIter;
use super::scrapers::CarsBG;
struct IdStream {
    futures: IntoIter<Pin<Box<dyn Future<Output = Result<Vec<String>, String>> + Send>>>,
}


impl IdStream {
    fn new(
        number_of_pages: u32,
        params: HashMap<String, String>,
        scraper: CarsBG,
        // other necessary parameters
    ) -> Self {
        
        let mut futures = Vec::new();
        for page in 1..=number_of_pages {
            
            // Explicitly type each future as `Box<dyn Future<...> + Send>`
            
            let future: Pin<Box<dyn Future<Output = Result<Vec<String>, String>> + Send>> =
                Box::pin(scraper.get_listed_ids(params.clone(), page));
            futures.push(future);
        }

        IdStream {
            futures: futures.into_iter(),
        }
    }
}

async fn fetch_ids(params: HashMap<String, String>, page_number: u32) -> Result<Vec<String>, String> {
    // Implement the logic to fetch and parse IDs from a page
    // ...

    Ok(vec![]) // Return the IDs from the page
}

impl Stream for IdStream {
    type Item = Result<String, String>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(mut future) = self.futures.next() {
            match future.poll_unpin(cx) {
                Poll::Ready(Ok(ids)) => {
                    if let Some(id) = ids.into_iter().next() {
                        // Yield the first ID and handle the rest as needed
                        Poll::Ready(Some(Ok(id)))
                    } else {
                        // No IDs found, continue to the next future
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                },
                Poll::Ready(Err(e)) => Poll::Ready(Some(Err(e))),
                Poll::Pending => Poll::Pending,
            }
        } else {
            // All futures have been processed
            Poll::Ready(None)
        }
    }
}

// Usage in an async function
pub async fn get_listed_ids(
    params: HashMap<String, String>,
    scraper: CarsBG,
) -> Result<Pin<Box<dyn Stream<Item = Result<String, String>> + Send>>, String> {
    let total_number = scraper.total_number(params.clone()).await?;
    let number_of_pages: u32 = ((total_number / 20) + 1)
        .try_into()
        .map_err(|_| "Failed to convert total number of pages to u32")?;
    
    let id_stream = IdStream::new(number_of_pages, params, scraper.clone());
    Ok(Box::pin(id_stream))
}
