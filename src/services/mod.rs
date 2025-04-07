pub mod CrawlerTrait;
pub mod JsonSearchBuider;
pub mod ScraperAppVehicleService;
pub mod SearchBuilder;
pub mod Searches;
pub mod VehicleService;
use std::sync::Arc;

use lazy_static::lazy_static;
use tokio::sync::{Mutex, Semaphore};

lazy_static! {
    static ref LINK_MUTEX: Mutex<()> = Mutex::new(());
    static ref SEMAPHORE: Arc<Semaphore> = Arc::new(Semaphore::new(4));
}
