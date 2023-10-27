use data_scraper::{cmd::change_log, config::app_config::AppConfig, utils::configure_log4rs};

#[tokio::main]
async fn main() {
    let app_config = AppConfig::from_file("config/config.yml");
    let logger_file_name = format!("{}/listing_log4rs.yml", app_config.get_log4rs_config());
    configure_log4rs(&logger_file_name);
    change_log(app_config.get_data_dir());
}
