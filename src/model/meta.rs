use log::info;
use serde::{Deserialize, Serialize};

use crate::{
    scraper::{
        agent::{get_header_data, get_pages},
        utils::extract_ascii_latin,
    },
    utils::listing_url,
    DATE_FORMAT,
};

use super::traits::{Header, Identity};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MetaHeader {
    pub timestamp: String,
    pub meta_type: String,
    pub make: String,
    pub model: String,
    pub total_number: u32,
    pub min_price: u32,
    pub max_price: u32,
    pub created_on: String,
    pub dealer: String,
}

impl Identity for MetaHeader {
    fn get_id(&self) -> String {
        self.timestamp.clone()
    }
}

impl Header for MetaHeader {
    fn header() -> Vec<&'static str> {
        vec![
            "timestamp",
            "dealer",
            "meta_type",
            "make",
            "model",
            "total_number",
            "min_price",
            "max_price",
            "created_on",
        ]
    }
}

impl MetaHeader {
    pub fn from_slink(slink: &str) -> Self {
        let url = &listing_url(slink, "1");
        let html = get_pages(url).unwrap();
        let content = get_header_data(&html).unwrap();
        let meta = extract_ascii_latin(&content);
        let re = regex::Regex::new(r" {2,}").unwrap();
        let split: Vec<&str> = re.split(meta.trim()).collect();
        let min_price = split[0].replace(' ', "").parse::<u32>().unwrap_or(0);
        let max_price = split[1].replace(' ', "").parse::<u32>().unwrap_or(0);
        let total_number = split[2].replace(' ', "").parse::<u32>().unwrap_or(0);
        if split.len() <= 4 {
            return MetaHeader {
                min_price,
                max_price,
                total_number,
                ..Default::default()
            };
        }

        let make_model: Vec<&str> = split[0].split_whitespace().collect();
        let (make, model) = if make_model.len() == 1 {
            (make_model[0], "")
        } else {
            (make_model[0], make_model[1])
        };

        MetaHeader {
            make: make.to_string(),
            model: model.to_string(),
            min_price,
            max_price,
            total_number,
            ..Default::default()
        }
    }

    pub fn from_string(raw: &str, meta_type: String, dealer: String) -> Self {
        let meta = extract_ascii_latin(raw);
        let re = regex::Regex::new(r" {2,}").unwrap();
        let split: Vec<&str> = re.split(meta.trim()).collect();
        for s in split.clone() {
            info!("split: {}", s);
        }
        let timestamp = chrono::Utc::now().timestamp().to_string();

        if split.len() <= 2 {
            return MetaHeader {
                ..Default::default()
            };
        }

        if split.len() <= 4 {
            let min_price = split[0].replace(' ', "").parse::<u32>().unwrap_or(0);
            let max_price = split[1].replace(' ', "").parse::<u32>().unwrap_or(0);
            let total_number = split[2].replace(' ', "").parse::<u32>().unwrap_or(0);
            return MetaHeader {
                timestamp,
                meta_type,
                make: "ALL".to_string(),
                model: "ALL".to_string(),
                min_price,
                max_price,
                total_number,
                created_on: chrono::Local::now().format(DATE_FORMAT).to_string(),
                dealer,
            };
        }

        let make_model: Vec<&str> = split[0].split_whitespace().collect();

        let (make, model) = if make_model.len() == 1 {
            (make_model[0], "")
        } else {
            (make_model[0], make_model[1])
        };

        let min = split[1].replace(' ', "").parse::<u32>().unwrap_or(0);
        let max = split[2].replace(' ', "").parse::<u32>().unwrap_or(0);
        let total_number = split[3].replace(' ', "").parse::<u32>().unwrap_or(0);

        MetaHeader {
            timestamp,
            meta_type,
            make: make.to_string(),
            model: model.to_string(),
            min_price: min,
            max_price: max,
            total_number,
            created_on: chrono::Local::now().format(DATE_FORMAT).to_string(),
            dealer,
        }
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
        scraper::agent::get_header_data, model::meta::MetaHeader, utils::configure_log4rs,
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
    fn test_read_skoda_meta_data() {
        let meta_content = get_header_data(HEADER_SKODA).unwrap();
        let meta = MetaHeader::from_string(&meta_content, "SELL".to_string(), "ALL".to_string());
        assert_eq!(meta.make, "Skoda");
        assert_eq!(meta.model, "Octavia");
        assert_eq!(meta.min_price, 2300);
        assert_eq!(meta.max_price, 9999);
        assert_eq!(meta.total_number, 13);
    }

    #[test]
    fn test_read_index_meta() {
        configure_log4rs("config/loggers/dev_log4rs.yml");
        info!("Test index meta");
        let meta_content = get_header_data(HEADER_INDEX).unwrap();
        let meta = MetaHeader::from_string(&meta_content, "SELL".to_string(), "ALL".to_string());
        assert_eq!(0, meta.total_number);
        assert_eq!(0, meta.max_price);
        assert_eq!(0, meta.min_price);
        let meta_content = get_header_data(EXT_SEARCH_HEADER).unwrap();
        let meta = MetaHeader::from_string(&meta_content, "SELL".to_string(), "ALL".to_string());
        assert_eq!(0, meta.total_number);
        assert_eq!(0, meta.max_price);
        assert_eq!(0, meta.min_price);
    }

    #[test]
    fn test_read_search_result_meta() {
        configure_log4rs("config/loggers/dev_log4rs.yml");
        info!("Test index meta");
        let meta_content = get_header_data(ALL_SEARCH_HEADER).unwrap();
        let meta = MetaHeader::from_string(&meta_content, "SELL".to_string(), "ALL".to_string());
        assert_eq!(83440, meta.total_number);
        assert_eq!(19557998, meta.max_price);
        assert_eq!(11, meta.min_price);
    }
}
