use data_scraper::{cmd::scrape_details, SEARCH_ALL};

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(scrape_details(SEARCH_ALL.slink.as_str()));
}
