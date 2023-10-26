use futures::executor::block_on;
use log::info;
use serde::{Deserialize, Serialize};

use crate::{
    scraper::{
        agent::{get_header_data, get_pages, get_pages_async, slink},
        utils::extract_ascii_latin,
    },
    utils::mobile_search_url,
    LISTING_URL, SEARCH_ALL, TIMESTAMP,
};

use super::{
    enums::{Dealer, SaleType},
    traits::{Header, Identity},
};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MetaHeader {
    pub slink: String,
    pub timestamp: i64,
    pub total_number: u32,
    pub min_price: u32,
    pub max_price: u32,
    pub dealer: Dealer,
    pub sale_type: SaleType,
}

impl Identity for MetaHeader {
    fn get_id(&self) -> String {
        self.timestamp.to_string()
    }
}

impl Header for MetaHeader {
    fn header() -> Vec<&'static str> {
        vec![
            "slink",
            "timestamp",
            "dealer",
            "sale_type",
            "total_number",
            "min_price",
            "max_price",
        ]
    }
}

pub fn statistic() -> Vec<MetaHeader> {
    let dealers_all = search(Dealer::DEALER, SaleType::NONE);
    let private_all = search(Dealer::PRIVATE, SaleType::NONE);
    return vec![SEARCH_ALL.clone(), dealers_all, private_all];
}

pub fn searches() -> Vec<MetaHeader> {
    let dealer_sold = search(Dealer::DEALER, SaleType::SOLD);
    let dealer_insale = search(Dealer::DEALER, SaleType::INSALE);
    let private_sold = search(Dealer::PRIVATE, SaleType::SOLD);
    let private_insale = search(Dealer::PRIVATE, SaleType::INSALE);
    return vec![dealer_sold, private_sold, dealer_insale, private_insale];
}

pub fn search(dealer_type: Dealer, sold: SaleType) -> MetaHeader {
    let metadata = block_on({
        let search_meta_data = asearch(dealer_type, sold);
        search_meta_data
    });
    return metadata;
}

pub async fn asearch(dealer_type: Dealer, sold: SaleType) -> MetaHeader {
    info!("Searching for: {:?} {:?}", dealer_type, sold);
    let url = mobile_search_url(LISTING_URL, "1", "", dealer_type, sold);
    info!("url: {}", url);
    let html = get_pages_async(&url).await.unwrap();
    // info!("content: {}", html);
    let slink = slink(&html);
    let content = get_header_data(&html).unwrap();
    let meta = extract_ascii_latin(&content);
    let re = regex::Regex::new(r" {2,}").unwrap();
    let split: Vec<&str> = re.split(meta.trim()).collect();
    info!("split: {:?}", split);
    let min_price = split[0].replace(' ', "").parse::<u32>().unwrap_or(0);
    let max_price = split[1].replace(' ', "").parse::<u32>().unwrap_or(0);
    let total_number = split[2].replace(' ', "").parse::<u32>().unwrap_or(0);
    return MetaHeader {
        slink,
        min_price,
        max_price,
        total_number,
        timestamp: *TIMESTAMP,
        dealer: dealer_type,
        sale_type: sold,
    };
}

impl MetaHeader {
    pub fn search(dealer_type: Dealer, sold: SaleType) -> Self {
        info!("Searching for: {:?} {:?}", dealer_type, sold);
        let url = mobile_search_url(LISTING_URL, "1", "", dealer_type, sold);
        info!("url: {}", url);
        let html = get_pages(&url).unwrap();
        // info!("content: {}", html);
        let slink = slink(&html);
        let content = get_header_data(&html).unwrap();
        let meta = extract_ascii_latin(&content);
        let re = regex::Regex::new(r" {2,}").unwrap();
        let split: Vec<&str> = re.split(meta.trim()).collect();
        info!("split: {:?}", split);
        let min_price = split[0].replace(' ', "").parse::<u32>().unwrap_or(0);
        let max_price = split[1].replace(' ', "").parse::<u32>().unwrap_or(0);
        let total_number = split[2].replace(' ', "").parse::<u32>().unwrap_or(0);
        return MetaHeader {
            slink,
            min_price,
            max_price,
            total_number,
            timestamp: *TIMESTAMP,
            dealer: dealer_type,
            sale_type: sold,
        };
    }

    pub fn page_numbers(&self) -> u32 {
        let mut pages = self.total_number / 20;
        if self.total_number % 20 > 0 {
            pages += 1;
        }
        pages
    }
}

#[cfg(test)]
mod test {
    use log::info;

    use crate::{
        model::{enums::Dealer, meta::MetaHeader},
        utils::configure_log4rs,
    };

    const HEADER_SKODA: &str = r#"
    <html lang="bg">
        <head>
            <script type="text/javascript" async="" src="https://static.criteo.net/js/ld/publishertag.prebid.117.js"></script><script type="text/javascript" async="" src="https://script.4dex.io/localstore.js"></script>
            <meta http-equiv="Content-Type" content="text/html; charset=windows-1251">
            <title>Skoda Octavia втора ръка и нови, обяви и цени — Mobile.bg</title>
            <meta name="description" content="Обяви за » Skoda Octavia » на цени започващи от 2 300 лв. до 9 999 лв. Избор от » 13 « предложения само в Mobile.bg">
        </head>
        <body></body>                
        </html>"#;
    const HEADER_INDEX: &str = r#"
        <html lang="bg">
            <head>
                <script type="text/javascript" async="" src="https://static.criteo.net/js/ld/publishertag.prebid.117.js"></script><script type="text/javascript" async="" src="https://script.4dex.io/localstore.js"></script>
                <meta http-equiv="Content-Type" content="text/html; charset=windows-1251">
                <title>Skoda Octavia втора ръка и нови, обяви и цени — Mobile.bg</title>
                <meta name="description" content=" 225 000 актуални обяви за продажба на автомобили, камиони, джипове, бусове, мотоциклети, селскостопанска и строителна техника, джетове,  авточасти и др. в Mobile.bg – сайтът за авто обяви.">
            </head>
            <body></body>                
            </html>"#;
    const EXT_SEARCH_HEADER: &str = r#"
    <html lang="bg">
        <head>
            <script type="text/javascript" async="" src="https://static.criteo.net/js/ld/publishertag.prebid.117.js"></script><script type="text/javascript" async="" src="https://script.4dex.io/localstore.js"></script>
            <meta http-equiv="Content-Type" content="text/html; charset=windows-1251">
            <title>Skoda Octavia втора ръка и нови, обяви и цени — Mobile.bg</title>
            <meta name="description" content="Безплатно търсене в хиляди обяви тип продава Автомобили и Джипове в Mobile.bg - най-големият сайт за Автомобили и Джипове">
        </head>
        <body></body>                
    </html>"#;

    const ALL_SEARCH_HEADER: &str = r#"
    <html lang="bg">
        <head>
            <script type="text/javascript" async="" src="https://static.criteo.net/js/ld/publishertag.prebid.117.js"></script><script type="text/javascript" async="" src="https://script.4dex.io/localstore.js"></script>
            <meta http-equiv="Content-Type" content="text/html; charset=windows-1251">
            <title>Skoda Octavia втора ръка и нови, обяви и цени — Mobile.bg</title>
            <meta name="description" content="Обяви за »  Автомобили и Джипове » на цени започващи от 11 лв. до 19 557 998 лв. Избор от » 83 440 « предложения само в Mobile.bg">
        </head>
        <body></body>                
    </html>"#;

    #[test]
    fn test_search() {
        configure_log4rs("config/loggers/dev_log4rs.yml");
        info!("Test index meta");
        let meta = MetaHeader::search(Dealer::DEALER, crate::model::enums::SaleType::SOLD);
        info!("meta: {:#?}", meta);
    }
}
