use data_scraper::cmd::scrape_listing;

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(scrape_listing());
}
