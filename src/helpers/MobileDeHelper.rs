use crate::model::MobileDe::MobileDeResults;

pub fn from_data(payload_data: &str) -> Result<MobileDeResults, String> {
    match serde_json::from_str::<MobileDeResults>(payload_data) {
        Ok(json) => Ok(json),
        Err(e) => Err(format!("Error parsing mobile.de listing: {:?}", e)),
    }
}

pub fn process_html(html: &str) -> Result<MobileDeResults, String> {
    let mut content = html.to_string();
    content = content.replace('\u{2009}', " ");
    content = content.replace('\u{a0}', "");
    content = content.replace("\u{003E}strong", "");
    content = content.replace('\u{003E}', "");
    content = content.replace('\u{003C}', "");

    if let Some(start_idx) = content.find("window.__INITIAL_STATE__ = ") {
        let start_idx = start_idx + "window.__INITIAL_STATE__ = ".len();
        if let Some(end_idx) = content.find("window.__PUBLIC_CONFIG__") {
            let json = &content[start_idx..end_idx];
            match serde_json::from_str::<MobileDeResults>(json) {
                Ok(json) => Ok(json),
                Err(e) => Err(format!("Error parsing mobile.de listing: {:?}", e)),
            }
        } else {
            Err("Error parsing mobile.de listing: end_idx not found".to_string())
        }
    } else {
        Err("Error parsing mobile.de listing: start_idx not found".to_string())
    }
}
