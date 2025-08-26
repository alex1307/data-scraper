use headless_chrome::{Browser, Tab};
use log::info;
use reqwest::get;
use serde::Deserialize;
use std::sync::Arc;

pub struct BrowserController {
    browser: Browser,
    tab: Arc<Tab>,
}

#[derive(Deserialize)]
struct DebuggerInfo {
    webSocketDebuggerUrl: String,
}
pub static HEADLESS_CHROME_URL: &str = "http://localhost:9223/json/version";
impl DebuggerInfo {
    async fn new() -> Result<Self, String> {
        let resp = get(HEADLESS_CHROME_URL)
            .await
            .map_err(|e| format!("Failed to fetch Chrome DevTools version: {}", e))?
            .json::<DebuggerInfo>()
            .await
            .map_err(|e| format!("Failed to parse debugger info: {}", e))?;

        info!("WebSocket URL: {}", resp.webSocketDebuggerUrl);
        Ok(resp)
    }
}

impl BrowserController {
    pub async fn new() -> Result<Self, String> {
        let debugger_info = DebuggerInfo::new().await?;
        let browser = Browser::connect(debugger_info.webSocketDebuggerUrl)
            .map_err(|e| format!("Browser connect error: {}", e))?;
        let tab = browser
            .new_tab()
            .map_err(|e| format!("Failed to create tab: {}", e))?;
        info!("New tab created");

        Ok(Self { browser, tab })
    }

    pub fn get_tab(&self) -> Arc<Tab> {
        self.tab.clone()
    }

    pub fn get_html(&self, url: &str) -> Result<String, String> {
        let tab = &self.tab;

        tab.navigate_to(url).map_err(|e| e.to_string())?;
        tab.wait_until_navigated().map_err(|e| e.to_string())?;

        tab.get_content().map_err(|e| e.to_string())
    }
}
#[cfg(test)]
mod tests {
    use crate::{LOG_CONFIG, utils::helpers::configure_log4rs};

    use super::*;

    #[tokio::test]
    async fn test_fetch_ws_url() {
        configure_log4rs(&LOG_CONFIG);
        let url = "https://www.autouncle.ro/en/cars_search?s%5Bmax_km%5D=200000&s%5Bmax_year%5D=2024&s%5Bmin_price%5D=90000&s%5Bmin_year%5D=2024&s%5Bnot_damaged%5D=true&s%5Bseller_kind%5D=Dealer&s%5Bwith_ratings%5D%5B%5D=5";
        let controller = BrowserController::new().await.unwrap();
        let html = controller.get_html(url).unwrap();
        assert!(html.contains("Autouncle"));
        assert!(html.contains("cars_search"));
        info!("HTML content: {}", html);
    }
}
