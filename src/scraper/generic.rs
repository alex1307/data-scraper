use std::collections::HashMap;


trait Scraper<T> where T: Clone + serde::Serialize + serde::de::DeserializeOwned {
    fn search_url(url: &str, query_params: HashMap<String, String>) -> String;
    fn total_number_of_pages(url: &str) -> u32;
    fn parse_listing(html: &str) -> Vec<HashMap<String, String>>;
    fn parse_details(url: &str) -> T;
    fn save2file(file_name: &str, data: Vec<T>);
}