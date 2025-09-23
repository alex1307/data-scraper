use crate::model::{Search::Search, VehicleDataModel::Vehicle};
use async_trait::async_trait;
use std::{fmt::Debug, sync::Arc};

use super::BrowserController::BrowserController;

#[async_trait]
pub trait VehicleScrapeTrait: Clone + Debug + Send + Sync + 'static {
    async fn process_listed_results(
        &self,
        search: Search,
        page: u32,
    ) -> Result<Vec<Vehicle>, String>;
    async fn get_html(&self, search: Search, page: u32) -> Result<String, String>;
    async fn browse_html(
        &self,
        browser: Arc<BrowserController>,
        search: Search,
        page: u32,
    ) -> Result<String, String>;

    fn total_number(&self, page: &str) -> Result<u32, String>;

    fn get_number_of_pages(&self, total_number: u32) -> Result<u32, String>;

    fn get_timeout(&self) -> u64 {
        250
    }

    fn get_search_url(&self, search: Search, page: u32) -> String {
        if page == 1 {
            return search.url;
        }
        format!("{}&page={}", search.url, page)
    }

    fn process_html(&self, html: &str) -> Result<Vec<Vehicle>, String>;
}
