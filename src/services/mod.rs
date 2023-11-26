pub mod ScraperAppService;
pub mod ScraperService;

use std::sync::Arc;

use lazy_static::lazy_static;
use tokio::sync::{Mutex, Semaphore};

lazy_static! {
    static ref LINK_MUTEX: Mutex<()> = Mutex::new(());
    static ref SEMAPHORE: Arc<Semaphore> = Arc::new(Semaphore::new(4));
}
