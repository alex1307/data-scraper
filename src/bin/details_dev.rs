use data_scraper::cmd::scrape_details;

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(scrape_details());
}
