
use data_scraper::mobile_scraper::{self, model::SearchRequest};
use encoding_rs::{UTF_8, WINDOWS_1251};

fn main() {
    let mut user_search_input = SearchRequest::new("Mercedes-Benz".to_string(), "C".to_string());
    user_search_input.set_from_year(2010);
    user_search_input.set_to_year(2011);
    let list_results = mobile_scraper::search_form_data("https://www.mobile.bg/pcgi/mobile.cgi", &user_search_input).unwrap();
    let links = mobile_scraper::get_links(list_results.as_ref()).unwrap();
    let meta_content = mobile_scraper::get_found_result(list_results.as_ref()).unwrap();
    println!("{}", meta_content);
    for link in links {
        println!("{}", link);
    }
    // let response = reqwest::blocking::get(
    //     "https://www.mobile.bg/pcgi/mobile.cgi?act=4&adv=11661885115343676&slink=ruvu7y",
    // )
    // .unwrap();

    // // Read the response body as a byte array
    // let bytes = response.bytes().unwrap().to_vec();

    // // Decode the byte array using the Windows-1251 encoding
    // let (html, _, _) = WINDOWS_1251.decode(&bytes);

    // // Convert the decoded text to UTF-8
    // let utf8_html = UTF_8.encode(&html).0;

    // // Print the UTF-8 encoded text to the console
    // println!("{}", String::from_utf8_lossy(&utf8_html));
    // let _ = mobile_scraper::parse_details(String::from_utf8_lossy(&utf8_html).as_ref());
    // let list = mobile_scraper::search().unwrap();
    // let links = mobile_scraper::get_links(list.as_ref()).unwrap();
    // for link in links {
    //     println!("{}", link);
    // }
}
