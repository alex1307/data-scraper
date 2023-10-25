use data_scraper::cmd::scrape;

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(scrape());
}
